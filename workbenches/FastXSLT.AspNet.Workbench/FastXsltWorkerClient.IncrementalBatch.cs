using System.Buffers.Binary;
using System.Diagnostics;
using System.Text;

public sealed partial class FastXsltWorkerClient
{
    private const byte VersionedIncrementalTransformBatch = 20;
    private const uint IncrementalBatchProtocolVersion = 1;
    private const byte BatchIncrementalEnd = 0x8c;
    private const byte VersionedBatchAcknowledged = 0x8d;

    public async Task<IsolatedWorkerIncrementalBatchResult> TransformIncrementalBatchAsync(
        IReadOnlyList<string> requestIdentities)
    {
        ArgumentOutOfRangeException.ThrowIfLessThan(requestIdentities.Count, 1);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(
            requestIdentities.Count,
            MaximumBatchMembers);
        await _gate.WaitAsync();
        try
        {
            ObjectDisposedException.ThrowIf(_disposed, this);
            var frame = BuildVersionedUncontrolledBatchFrame(requestIdentities);
            return await SendIncrementalFrameLockedAsync(
                frame,
                requestIdentities,
                TimeSpan.Zero,
                null,
                retainOutcomes: true);
        }
        finally
        {
            _gate.Release();
        }
    }

    public async Task<FastXsltWorkerException>
        ProbeUnsupportedIncrementalBatchProtocolVersionAsync()
    {
        await _gate.WaitAsync();
        try
        {
            ObjectDisposedException.ThrowIf(_disposed, this);
            var frame = BuildVersionedIncrementalEnvelope(
                [0xff, 0xff, 0xff],
                IncrementalBatchProtocolVersion + 1);
            await _input.WriteAsync(frame);
            await _input.FlushAsync();
            if (await ReadByteAsync() != Error)
            {
                throw new InvalidDataException(
                    "Unsupported incremental batch protocol version was admitted.");
            }
            return await ReadFailureAsync();
        }
        finally
        {
            _gate.Release();
        }
    }

    public async Task<IsolatedWorkerIncrementalBatchResult>
        TransformControlledIncrementalBatchAsync(
            IReadOnlyList<IsolatedWorkerBatchRequest> requests)
    {
        ArgumentOutOfRangeException.ThrowIfLessThan(requests.Count, 1);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(requests.Count, MaximumBatchMembers);
        await _gate.WaitAsync();
        try
        {
            ObjectDisposedException.ThrowIf(_disposed, this);
            var frame = BuildVersionedControlledBatchFrame(requests);
            return await SendIncrementalFrameLockedAsync(
                frame,
                requests.Select(static request => request.RequestIdentity).ToArray(),
                TimeSpan.Zero,
                null,
                retainOutcomes: true);
        }
        finally
        {
            _gate.Release();
        }
    }

    private async Task<IsolatedWorkerIncrementalBatchResult> SendIncrementalFrameLockedAsync(
        byte[] frame,
        IReadOnlyList<string> requestIdentities,
        TimeSpan outcomeReadDelay,
        Func<IsolatedWorkerBatchOutcome, ValueTask>? consumer,
        bool retainOutcomes)
    {
        var started = Stopwatch.GetTimestamp();
        var outcomes = retainOutcomes
            ? new List<IsolatedWorkerBatchOutcome>(requestIdentities.Count)
            : null;
        var acknowledged = false;
        var completeOutcomeCount = 0;
        var consumerAbandoned = false;
        var encodedResponseBytes = 0;
        var retainedOutcomeWireBytes = 0;
        var peakRetainedOutcomeWireBytes = 0;
        TimeSpan? firstOutcomeElapsed = null;
        try
        {
            await _input.WriteAsync(frame);
            await _input.FlushAsync();

            var response = await ReadByteAsync();
            if (response == Error)
            {
                throw await ReadFailureAsync();
            }
            if (response != VersionedBatchAcknowledged ||
                await ReadUInt32Async() != IncrementalBatchProtocolVersion ||
                await ReadUInt32Async() != (uint)requestIdentities.Count)
            {
                throw new InvalidDataException("Incremental batch acknowledgement changed.");
            }
            acknowledged = true;
            encodedResponseBytes = 1 + sizeof(uint) + sizeof(uint);

            for (var expectedIndex = 0;
                expectedIndex < requestIdentities.Count;
                expectedIndex++)
            {
                response = await ReadByteAsync();
                if (response == Error)
                {
                    throw new IsolatedWorkerIncrementalBatchTerminatedException(
                        await ReadFailureAsync(),
                        outcomes ?? [],
                        completeOutcomeCount,
                        expectedIndex,
                        requestIdentities.Count);
                }
                if (response != BatchIncrementalOutcome ||
                    await ReadUInt32Async() != (uint)expectedIndex)
                {
                    throw new InvalidDataException("Incremental batch outcome position changed.");
                }

                var outcome = await ReadIncrementalOutcomeAsync(
                    requestIdentities[expectedIndex]);
                var outcomeWireBytes = EncodedIncrementalOutcomeBytes(outcome);
                encodedResponseBytes = checked(encodedResponseBytes + outcomeWireBytes);
                retainedOutcomeWireBytes = retainOutcomes
                    ? checked(retainedOutcomeWireBytes + outcomeWireBytes)
                    : outcomeWireBytes;
                peakRetainedOutcomeWireBytes = Math.Max(
                    peakRetainedOutcomeWireBytes,
                    retainedOutcomeWireBytes);
                outcomes?.Add(outcome);
                try
                {
                    await DeliverOutcomeAsync(consumer, outcome);
                }
                catch
                {
                    consumerAbandoned = true;
                    throw;
                }
                completeOutcomeCount++;
                if (!retainOutcomes)
                {
                    retainedOutcomeWireBytes = 0;
                }
                firstOutcomeElapsed ??= Stopwatch.GetElapsedTime(started);
                if (outcomeReadDelay > TimeSpan.Zero &&
                    expectedIndex + 1 < requestIdentities.Count)
                {
                    await Task.Delay(outcomeReadDelay);
                }
            }

            if (await ReadByteAsync() != BatchIncrementalEnd ||
                await ReadUInt32Async() != (uint)requestIdentities.Count)
            {
                throw new InvalidDataException("Incremental batch terminal frame changed.");
            }
            encodedResponseBytes = checked(encodedResponseBytes + 1 + sizeof(uint));
        }
        catch (IOException failure) when (!consumerAbandoned)
        {
            TerminateForExperiment();
            throw new IsolatedWorkerIncrementalBatchTransportException(
                failure,
                acknowledged,
                outcomes ?? [],
                completeOutcomeCount,
                requestIdentities.Count,
                frame.Length,
                encodedResponseBytes,
                peakRetainedOutcomeWireBytes);
        }
        return new IsolatedWorkerIncrementalBatchResult(
            outcomes ?? [],
            requestIdentities.Count,
            firstOutcomeElapsed ?? Stopwatch.GetElapsedTime(started),
            Stopwatch.GetElapsedTime(started),
            frame.Length,
            encodedResponseBytes,
            retainOutcomes ? requestIdentities.Count : 1,
            peakRetainedOutcomeWireBytes);
    }

    private static int EncodedIncrementalOutcomeBytes(IsolatedWorkerBatchOutcome outcome)
    {
        var fields = 1;
        if (outcome.Failure is null)
        {
            fields = checked(fields + EncodedStringBytes(outcome.RequestIdentity));
            fields = checked(fields + EncodedStringBytes(outcome.Result!));
        }
        else
        {
            var failure = outcome.Failure;
            fields = checked(fields + EncodedStringBytes(failure.Code));
            fields = checked(fields + EncodedStringBytes(failure.Category));
            fields = checked(fields + EncodedStringBytes(failure.RequestId ?? string.Empty));
            fields = checked(fields + EncodedStringBytes(
                failure.Location?.Resource ?? string.Empty));
            fields = checked(fields + EncodedStringBytes(
                failure.Location?.Start.ToString(System.Globalization.CultureInfo.InvariantCulture)
                    ?? string.Empty));
            fields = checked(fields + EncodedStringBytes(
                failure.Location?.End.ToString(System.Globalization.CultureInfo.InvariantCulture)
                    ?? string.Empty));
            fields = checked(fields + EncodedStringBytes(failure.Detail));
        }
        return checked(1 + sizeof(uint) + fields);
    }

    private static int EncodedStringBytes(string value) =>
        checked(sizeof(int) + Encoding.UTF8.GetByteCount(value));

    private async Task<IsolatedWorkerBatchOutcome> ReadIncrementalOutcomeAsync(
        string expectedIdentity)
    {
        var outcomeKind = await ReadByteAsync();
        if (outcomeKind == Result)
        {
            var identity = await ReadStringAsync();
            RequireBatchIdentity(expectedIdentity, identity);
            return new IsolatedWorkerBatchOutcome(identity, await ReadStringAsync(), null);
        }
        if (outcomeKind == Error)
        {
            var failure = await ReadFailureAsync();
            RequireBatchIdentity(expectedIdentity, failure.RequestId);
            return new IsolatedWorkerBatchOutcome(failure.RequestId!, null, failure);
        }
        throw new InvalidDataException($"Unexpected incremental batch outcome: {outcomeKind}.");
    }

    private async ValueTask DeliverOutcomeAsync(
        Func<IsolatedWorkerBatchOutcome, ValueTask>? consumer,
        IsolatedWorkerBatchOutcome outcome)
    {
        if (consumer is null)
        {
            return;
        }
        try
        {
            await consumer(outcome);
        }
        catch
        {
            TerminateForExperiment();
            throw;
        }
    }

    public async Task<IsolatedWorkerIncrementalBatchResult> ConsumeIncrementalBatchAsync(
        IReadOnlyList<string> requestIdentities,
        Func<IsolatedWorkerBatchOutcome, ValueTask> consumer)
    {
        ArgumentNullException.ThrowIfNull(consumer);
        ArgumentOutOfRangeException.ThrowIfLessThan(requestIdentities.Count, 1);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(
            requestIdentities.Count,
            MaximumBatchMembers);
        await _gate.WaitAsync();
        try
        {
            ObjectDisposedException.ThrowIf(_disposed, this);
            return await SendIncrementalFrameLockedAsync(
                BuildVersionedUncontrolledBatchFrame(requestIdentities),
                requestIdentities,
                TimeSpan.Zero,
                consumer,
                retainOutcomes: false);
        }
        finally
        {
            _gate.Release();
        }
    }

    public async Task<IsolatedWorkerIncrementalBatchResult>
        TransformIncrementalBatchWithReadDelayAsync(
            IReadOnlyList<string> requestIdentities,
            TimeSpan outcomeReadDelay)
    {
        ArgumentOutOfRangeException.ThrowIfLessThan(requestIdentities.Count, 1);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(
            requestIdentities.Count,
            MaximumBatchMembers);
        ArgumentOutOfRangeException.ThrowIfLessThan(outcomeReadDelay, TimeSpan.Zero);
        await _gate.WaitAsync();
        try
        {
            ObjectDisposedException.ThrowIf(_disposed, this);
            return await SendIncrementalFrameLockedAsync(
                BuildVersionedUncontrolledBatchFrame(requestIdentities),
                requestIdentities,
                outcomeReadDelay,
                null,
                retainOutcomes: true);
        }
        finally
        {
            _gate.Release();
        }
    }

    private static byte[] BuildUncontrolledBatchFrame(
        byte operation,
        IReadOnlyList<string> requestIdentities)
    {
        var encodedLength = checked(1 + sizeof(int));
        foreach (var identity in requestIdentities)
        {
            encodedLength = checked(
                encodedLength + sizeof(int) + Encoding.UTF8.GetByteCount(identity));
        }
        if (encodedLength > MaximumFrameBytes)
        {
            throw new InvalidDataException(
                $"Batch frame exceeds {MaximumFrameBytes} bytes.");
        }

        var frame = new byte[encodedLength];
        frame[0] = operation;
        BinaryPrimitives.WriteInt32LittleEndian(
            frame.AsSpan(1, sizeof(int)),
            requestIdentities.Count);
        var offset = 1 + sizeof(int);
        foreach (var identity in requestIdentities)
        {
            var byteCount = Encoding.UTF8.GetByteCount(identity);
            BinaryPrimitives.WriteInt32LittleEndian(
                frame.AsSpan(offset, sizeof(int)),
                byteCount);
            offset += sizeof(int);
            offset += Encoding.UTF8.GetBytes(identity, frame.AsSpan(offset, byteCount));
        }
        return frame;
    }

    private static byte[] BuildVersionedUncontrolledBatchFrame(
        IReadOnlyList<string> requestIdentities)
    {
        var controlledShape = requestIdentities
            .Select(static identity => new IsolatedWorkerBatchRequest(identity))
            .ToArray();
        return BuildVersionedControlledBatchFrame(controlledShape);
    }

    private static byte[] BuildControlledBatchFrame(
        byte operation,
        IReadOnlyList<IsolatedWorkerBatchRequest> requests)
    {
        var encodedLength = checked(1 + sizeof(int));
        foreach (var request in requests)
        {
            if (request.Cancelled && request.MaximumXsltInstructions is not null)
            {
                throw new ArgumentException(
                    "A batch member cannot combine pre-cancellation and an instruction limit.",
                    nameof(requests));
            }
            encodedLength = checked(
                encodedLength
                    + sizeof(int)
                    + Encoding.UTF8.GetByteCount(request.RequestIdentity)
                    + 2
                    + (request.MaximumXsltInstructions is null ? 0 : sizeof(ulong)));
        }
        if (encodedLength > MaximumFrameBytes)
        {
            throw new InvalidDataException($"Batch frame exceeds {MaximumFrameBytes} bytes.");
        }

        var frame = new byte[encodedLength];
        frame[0] = operation;
        BinaryPrimitives.WriteInt32LittleEndian(frame.AsSpan(1, sizeof(int)), requests.Count);
        var offset = 1 + sizeof(int);
        foreach (var request in requests)
        {
            var byteCount = Encoding.UTF8.GetByteCount(request.RequestIdentity);
            BinaryPrimitives.WriteInt32LittleEndian(
                frame.AsSpan(offset, sizeof(int)),
                byteCount);
            offset += sizeof(int);
            offset += Encoding.UTF8.GetBytes(
                request.RequestIdentity,
                frame.AsSpan(offset, byteCount));
            frame[offset++] = request.Cancelled ? (byte)1 : (byte)0;
            frame[offset++] = request.MaximumXsltInstructions is null ? (byte)0 : (byte)1;
            if (request.MaximumXsltInstructions is ulong maximum)
            {
                BinaryPrimitives.WriteUInt64LittleEndian(
                    frame.AsSpan(offset, sizeof(ulong)),
                    maximum);
                offset += sizeof(ulong);
            }
        }
        return frame;
    }

    private static byte[] BuildVersionedControlledBatchFrame(
        IReadOnlyList<IsolatedWorkerBatchRequest> requests)
    {
        var unversioned = BuildControlledBatchFrame(0, requests);
        return BuildVersionedIncrementalEnvelope(unversioned.AsSpan(1));
    }

    private static byte[] BuildVersionedIncrementalEnvelope(
        ReadOnlySpan<byte> payload,
        uint protocolVersion = IncrementalBatchProtocolVersion)
    {
        var encodedLength = checked(1 + sizeof(uint) + sizeof(int) + payload.Length);
        if (encodedLength > MaximumFrameBytes)
        {
            throw new InvalidDataException(
                $"Batch frame exceeds {MaximumFrameBytes} bytes.");
        }

        var frame = new byte[encodedLength];
        frame[0] = VersionedIncrementalTransformBatch;
        BinaryPrimitives.WriteUInt32LittleEndian(
            frame.AsSpan(1, sizeof(uint)),
            protocolVersion);
        BinaryPrimitives.WriteInt32LittleEndian(
            frame.AsSpan(1 + sizeof(uint), sizeof(int)),
            payload.Length);
        payload.CopyTo(frame.AsSpan(1 + sizeof(uint) + sizeof(int)));
        return frame;
    }
}

public sealed record IsolatedWorkerIncrementalBatchResult(
    IReadOnlyList<IsolatedWorkerBatchOutcome> Outcomes,
    int CompleteOutcomeCount,
    TimeSpan FirstOutcomeElapsed,
    TimeSpan FinalOutcomeElapsed,
    int EncodedRequestBytes,
    int EncodedResponseBytes,
    int PeakAdapterRetainedOutcomeCount,
    int PeakAdapterRetainedOutcomeWireBytes);

public sealed class IsolatedWorkerIncrementalBatchTerminatedException : Exception
{
    public IsolatedWorkerIncrementalBatchTerminatedException(
        FastXsltWorkerException failure,
        IReadOnlyList<IsolatedWorkerBatchOutcome> completeOutcomes,
        int completeOutcomeCount,
        int ambiguousMemberIndex,
        int memberCount)
        : base(failure.Message, failure)
    {
        Failure = failure;
        CompleteOutcomes = completeOutcomes;
        CompleteOutcomeCount = completeOutcomeCount;
        AmbiguousMemberIndex = ambiguousMemberIndex;
        MemberCount = memberCount;
    }

    public FastXsltWorkerException Failure { get; }
    public IReadOnlyList<IsolatedWorkerBatchOutcome> CompleteOutcomes { get; }
    public int CompleteOutcomeCount { get; }
    public int AmbiguousMemberIndex { get; }
    public int MemberCount { get; }
    public int UnstartedMemberCount => MemberCount - AmbiguousMemberIndex - 1;
}

public sealed class IsolatedWorkerIncrementalBatchTransportException : IOException
{
    public IsolatedWorkerIncrementalBatchTransportException(
        IOException failure,
        bool acknowledged,
        IReadOnlyList<IsolatedWorkerBatchOutcome> completeOutcomes,
        int completeOutcomeCount,
        int memberCount,
        int encodedRequestBytes,
        int encodedCompleteResponseBytes,
        int peakAdapterRetainedOutcomeWireBytes)
        : base("Incremental batch transport ended before its terminal frame.", failure)
    {
        Acknowledged = acknowledged;
        CompleteOutcomes = completeOutcomes;
        CompleteOutcomeCount = completeOutcomeCount;
        MemberCount = memberCount;
        EncodedRequestBytes = encodedRequestBytes;
        EncodedCompleteResponseBytes = encodedCompleteResponseBytes;
        PeakAdapterRetainedOutcomeWireBytes = peakAdapterRetainedOutcomeWireBytes;
    }

    public bool Acknowledged { get; }
    public IReadOnlyList<IsolatedWorkerBatchOutcome> CompleteOutcomes { get; }
    public int CompleteOutcomeCount { get; }
    public int MemberCount { get; }
    public int EncodedRequestBytes { get; }
    public int EncodedCompleteResponseBytes { get; }
    public int PeakAdapterRetainedOutcomeWireBytes { get; }
    public int RemainingAmbiguousCount => MemberCount - CompleteOutcomeCount;
}
