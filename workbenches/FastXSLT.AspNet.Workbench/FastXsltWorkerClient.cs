using System.Buffers.Binary;
using System.Diagnostics;
using System.Text;

public sealed partial class FastXsltWorkerClient : IDisposable
{
    private const byte Initialize = 1;
    private const byte Transform = 2;
    private const byte Shutdown = 3;
    private const byte NonCooperatingProbe = 4;
    private const byte CancelledTransform = 5;
    private const byte ControlledTransform = 6;
    private const byte Cancel = 7;
    private const byte UnpausedControlledTransform = 8;
    private const byte InstructionLimitedTransform = 9;
    private const byte InitializeWithStylesheetDependency = 10;
    private const byte MeasuredTransform = 11;
    private const byte TransformBatch = 12;
    private const byte ControlledTransformBatch = 13;
    private const byte BatchLossProbe = 14;
    private const byte BatchTransferLossProbe = 15;
    private const byte ActiveCancellationBatch = 16;
    private const byte NaturalCancellationBatch = 17;
    private const byte Ready = 0x81;
    private const byte Result = 0x82;
    private const byte Stopped = 0x83;
    private const byte ProbeStarted = 0x84;
    private const byte TransformStarted = 0x85;
    private const byte MeasuredResult = 0x86;
    private const byte BatchResult = 0x87;
    private const byte BatchAcknowledged = 0x88;
    private const byte BatchMemberStarted = 0x89;
    private const byte BatchMemberFinished = 0x8a;
    private const byte BatchIncrementalOutcome = 0x8b;
    private const byte Error = 0xff;
    private const int MaximumFrameBytes = 1_048_576;
    private const int MaximumBatchMembers = 128;

    private readonly Process _process;
    private readonly Stream _input;
    private readonly Stream _output;
    private readonly SemaphoreSlim _gate = new(1, 1);
    private readonly SerializedWorkerControlWriter _controlWriter;
    private bool _disposed;

    private FastXsltWorkerClient(Process process)
    {
        _process = process;
        _input = process.StandardInput.BaseStream;
        _output = process.StandardOutput.BaseStream;
        _controlWriter = new SerializedWorkerControlWriter(_input, MaximumFrameBytes);
        process.BeginErrorReadLine();
    }

    public static async Task<FastXsltWorkerClient> StartAsync(
        string workerPath,
        string sourceIdentity,
        byte[] source,
        string stylesheetIdentity,
        byte[] stylesheet)
        => await StartCoreAsync(
            workerPath,
            sourceIdentity,
            source,
            stylesheetIdentity,
            stylesheet,
            null,
            null,
            admitted: false,
            denied: false);

    public static async Task<FastXsltWorkerClient> StartWithStylesheetDependencyAsync(
        string workerPath,
        string sourceIdentity,
        byte[] source,
        string stylesheetIdentity,
        byte[] stylesheet,
        string dependencyIdentity,
        byte[] dependency,
        bool admitted,
        bool denied)
    {
        if (!admitted && dependency.Length != 0)
        {
            throw new ArgumentException(
                "An unadmitted stylesheet dependency must not carry bytes.",
                nameof(dependency));
        }
        return await StartCoreAsync(
            workerPath,
            sourceIdentity,
            source,
            stylesheetIdentity,
            stylesheet,
            dependencyIdentity,
            dependency,
            admitted,
            denied);
    }

    private static async Task<FastXsltWorkerClient> StartCoreAsync(
        string workerPath,
        string sourceIdentity,
        byte[] source,
        string stylesheetIdentity,
        byte[] stylesheet,
        string? dependencyIdentity,
        byte[]? dependency,
        bool admitted,
        bool denied)
    {
        if (!File.Exists(workerPath))
        {
            throw new FileNotFoundException(
                "Build the isolated worker with `cargo build --release -p fastxslt-worker`.",
                workerPath);
        }
        var process = Process.Start(new ProcessStartInfo(workerPath)
        {
            RedirectStandardInput = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
            CreateNoWindow = true
        }) ?? throw new InvalidOperationException("Failed to start the FastXSLT worker.");
        var client = new FastXsltWorkerClient(process);
        try
        {
            await client.WriteByteAsync(
                dependencyIdentity is null ? Initialize : InitializeWithStylesheetDependency);
            await client.WriteStringAsync(sourceIdentity);
            await client.WriteBytesAsync(source);
            await client.WriteStringAsync(stylesheetIdentity);
            await client.WriteBytesAsync(stylesheet);
            if (dependencyIdentity is not null)
            {
                await client.WriteStringAsync(dependencyIdentity);
                await client.WriteBytesAsync(dependency ?? []);
                await client.WriteByteAsync(admitted ? (byte)1 : (byte)0);
                await client.WriteByteAsync(denied ? (byte)1 : (byte)0);
            }
            await client._input.FlushAsync();
            var response = await client.ReadByteAsync();
            if (response == Error)
            {
                throw await client.ReadFailureAsync();
            }
            if (response != Ready)
            {
                throw new InvalidDataException($"Unexpected initialization response: {response}.");
            }
            return client;
        }
        catch
        {
            client.Dispose();
            throw;
        }
    }

    public async Task<string> TransformAsync(string requestIdentity)
    {
        await _gate.WaitAsync();
        try
        {
            ObjectDisposedException.ThrowIf(_disposed, this);
            await WriteByteAsync(Transform);
            await WriteStringAsync(requestIdentity);
            await _input.FlushAsync();
            var response = await ReadByteAsync();
            if (response == Error)
            {
                throw await ReadFailureAsync();
            }
            if (response != Result)
            {
                throw new InvalidDataException($"Unexpected transform response: {response}.");
            }
            var correlatedIdentity = await ReadStringAsync();
            if (!StringComparer.Ordinal.Equals(requestIdentity, correlatedIdentity))
            {
                throw new InvalidDataException("Worker response identity did not match the request.");
            }
            return await ReadStringAsync();
        }
        finally
        {
            _gate.Release();
        }
    }

    public async Task<IsolatedWorkerMeasuredTransform> TransformMeasuredAsync(
        string requestIdentity)
    {
        var totalStarted = Stopwatch.GetTimestamp();
        var gateStarted = totalStarted;
        await _gate.WaitAsync();
        var gateElapsed = Stopwatch.GetElapsedTime(gateStarted);
        try
        {
            ObjectDisposedException.ThrowIf(_disposed, this);

            var writeStarted = Stopwatch.GetTimestamp();
            await WriteByteAsync(MeasuredTransform);
            await WriteStringAsync(requestIdentity);
            var writeElapsed = Stopwatch.GetElapsedTime(writeStarted);

            var flushStarted = Stopwatch.GetTimestamp();
            await _input.FlushAsync();
            var flushElapsed = Stopwatch.GetElapsedTime(flushStarted);

            var responseStarted = Stopwatch.GetTimestamp();
            var response = await ReadByteAsync();
            if (response == Error)
            {
                throw await ReadFailureAsync();
            }
            if (response != MeasuredResult)
            {
                throw new InvalidDataException($"Unexpected measured transform response: {response}.");
            }
            var correlatedIdentity = await ReadStringAsync();
            if (!StringComparer.Ordinal.Equals(requestIdentity, correlatedIdentity))
            {
                throw new InvalidDataException("Worker response identity did not match the request.");
            }
            var result = await ReadStringAsync();
            var workerDecodeNanoseconds = await ReadUInt64Async();
            var workerQueueNanoseconds = await ReadUInt64Async();
            var workerExecutionNanoseconds = await ReadUInt64Async();
            var responseElapsed = Stopwatch.GetElapsedTime(responseStarted);
            var totalElapsed = Stopwatch.GetElapsedTime(totalStarted);

            return new IsolatedWorkerMeasuredTransform(
                result,
                new IsolatedWorkerTransformTiming(
                    gateElapsed.TotalMicroseconds,
                    writeElapsed.TotalMicroseconds,
                    flushElapsed.TotalMicroseconds,
                    responseElapsed.TotalMicroseconds,
                    workerDecodeNanoseconds / 1_000.0,
                    workerQueueNanoseconds / 1_000.0,
                    workerExecutionNanoseconds / 1_000.0,
                    totalElapsed.TotalMicroseconds));
        }
        finally
        {
            _gate.Release();
        }
    }

    public async Task<IReadOnlyList<IsolatedWorkerBatchOutcome>> TransformBatchAsync(
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
            var frame = BuildUncontrolledBatchFrame(TransformBatch, requestIdentities);
            return await SendBatchFrameLockedAsync(frame, requestIdentities);
        }
        finally
        {
            _gate.Release();
        }
    }

    public async Task<IReadOnlyList<IsolatedWorkerBatchOutcome>> TransformControlledBatchAsync(
        IReadOnlyList<IsolatedWorkerBatchRequest> requests)
    {
        ArgumentOutOfRangeException.ThrowIfLessThan(requests.Count, 1);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(requests.Count, MaximumBatchMembers);
        await _gate.WaitAsync();
        try
        {
            ObjectDisposedException.ThrowIf(_disposed, this);
            var frame = BuildControlledBatchFrame(ControlledTransformBatch, requests);
            return await SendBatchFrameLockedAsync(
                frame,
                requests.Select(static request => request.RequestIdentity).ToArray());
        }
        finally
        {
            _gate.Release();
        }
    }

    public async Task<IReadOnlyList<IsolatedBatchLossObservation>> ReachBatchLossBarrierAsync(
        IReadOnlyList<string> requestIdentities,
        int parkAtIndex)
    {
        ArgumentOutOfRangeException.ThrowIfLessThan(requestIdentities.Count, 1);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(
            requestIdentities.Count,
            MaximumBatchMembers);
        ArgumentOutOfRangeException.ThrowIfNegative(parkAtIndex);
        ArgumentOutOfRangeException.ThrowIfGreaterThanOrEqual(
            parkAtIndex,
            requestIdentities.Count);

        await _gate.WaitAsync();
        try
        {
            ObjectDisposedException.ThrowIf(_disposed, this);
            var encodedLength = checked(1 + sizeof(int) + sizeof(int));
            foreach (var identity in requestIdentities)
            {
                encodedLength = checked(
                    encodedLength + sizeof(int) + Encoding.UTF8.GetByteCount(identity));
            }
            if (encodedLength > MaximumFrameBytes)
            {
                throw new InvalidDataException(
                    $"Batch loss-probe frame exceeds {MaximumFrameBytes} bytes.");
            }

            var frame = new byte[encodedLength];
            frame[0] = BatchLossProbe;
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
            BinaryPrimitives.WriteInt32LittleEndian(
                frame.AsSpan(offset, sizeof(int)),
                parkAtIndex);

            await _input.WriteAsync(frame);
            await _input.FlushAsync();
            var response = await ReadByteAsync();
            if (response == Error)
            {
                throw await ReadFailureAsync();
            }
            if (response != BatchAcknowledged)
            {
                throw new InvalidDataException(
                    $"Unexpected batch loss-probe acknowledgement: {response}.");
            }
            var acknowledgedCount = await ReadUInt32Async();
            if (acknowledgedCount != (uint)requestIdentities.Count)
            {
                throw new InvalidDataException(
                    "Worker acknowledged a different batch member count.");
            }

            var observations = new List<IsolatedBatchLossObservation>();
            while (true)
            {
                var phase = await ReadByteAsync();
                if (phase != BatchMemberStarted && phase != BatchMemberFinished)
                {
                    throw new InvalidDataException(
                        $"Unexpected batch loss-probe observation: {phase}.");
                }
                var index = checked((int)await ReadUInt32Async());
                if (index < 0 || index >= requestIdentities.Count)
                {
                    throw new InvalidDataException(
                        $"Batch loss-probe member index {index} is outside the request.");
                }
                var identity = await ReadStringAsync();
                RequireBatchIdentity(requestIdentities[index], identity);
                var observation = new IsolatedBatchLossObservation(
                    index,
                    identity,
                    phase == BatchMemberStarted ? "started" : "finished");
                observations.Add(observation);
                if (phase == BatchMemberStarted && index == parkAtIndex)
                {
                    return observations;
                }
            }
        }
        finally
        {
            _gate.Release();
        }
    }

    public async Task<IsolatedBatchTransferLossProbe> ReachBatchTransferLossBarrierAsync(
        IReadOnlyList<string> requestIdentities,
        int truncateAtIndex)
    {
        ArgumentOutOfRangeException.ThrowIfLessThan(requestIdentities.Count, 1);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(
            requestIdentities.Count,
            MaximumBatchMembers);
        ArgumentOutOfRangeException.ThrowIfNegative(truncateAtIndex);
        ArgumentOutOfRangeException.ThrowIfGreaterThanOrEqual(
            truncateAtIndex,
            requestIdentities.Count);

        await _gate.WaitAsync();
        try
        {
            ObjectDisposedException.ThrowIf(_disposed, this);
            var encodedLength = checked(1 + sizeof(int) + sizeof(int));
            foreach (var identity in requestIdentities)
            {
                encodedLength = checked(
                    encodedLength + sizeof(int) + Encoding.UTF8.GetByteCount(identity));
            }
            if (encodedLength > MaximumFrameBytes)
            {
                throw new InvalidDataException(
                    $"Batch transfer-loss probe exceeds {MaximumFrameBytes} bytes.");
            }

            var frame = new byte[encodedLength];
            frame[0] = BatchTransferLossProbe;
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
            BinaryPrimitives.WriteInt32LittleEndian(
                frame.AsSpan(offset, sizeof(int)),
                truncateAtIndex);

            await _input.WriteAsync(frame);
            await _input.FlushAsync();
            if (await ReadByteAsync() != BatchAcknowledged)
            {
                throw new InvalidDataException(
                    "Worker did not acknowledge the batch transfer-loss probe.");
            }
            if (await ReadUInt32Async() != (uint)requestIdentities.Count)
            {
                throw new InvalidDataException(
                    "Worker acknowledged a different transfer-loss member count.");
            }

            var complete = new List<IsolatedWorkerBatchOutcome>(truncateAtIndex);
            for (var expectedIndex = 0; expectedIndex <= truncateAtIndex; expectedIndex++)
            {
                if (await ReadByteAsync() != BatchIncrementalOutcome)
                {
                    throw new InvalidDataException(
                        "Worker did not emit the expected incremental probe outcome.");
                }
                var actualIndex = checked((int)await ReadUInt32Async());
                if (actualIndex != expectedIndex || await ReadByteAsync() != Result)
                {
                    throw new InvalidDataException(
                        "Worker transfer-loss outcome position or kind changed.");
                }
                var identity = await ReadStringAsync();
                RequireBatchIdentity(requestIdentities[expectedIndex], identity);
                if (expectedIndex < truncateAtIndex)
                {
                    complete.Add(new IsolatedWorkerBatchOutcome(
                        identity,
                        await ReadStringAsync(),
                        null));
                    continue;
                }

                var declaredResultBytes = checked((int)await ReadUInt32Async());
                if (declaredResultBytes < 2 || declaredResultBytes > MaximumFrameBytes)
                {
                    throw new InvalidDataException(
                        $"Invalid transfer-loss result length: {declaredResultBytes}.");
                }
                var observedResultBytes = declaredResultBytes / 2;
                var partial = new byte[observedResultBytes];
                await _output.ReadExactlyAsync(partial);
                return new IsolatedBatchTransferLossProbe(
                    complete,
                    truncateAtIndex,
                    identity,
                    declaredResultBytes,
                    observedResultBytes);
            }
            throw new InvalidDataException("Transfer-loss probe did not reach its barrier.");
        }
        finally
        {
            _gate.Release();
        }
    }

    public async Task<int> SendTruncatedBatchCommandForExperimentAsync()
    {
        await _gate.WaitAsync();
        try
        {
            ObjectDisposedException.ThrowIf(_disposed, this);
            var frame = new byte[
                1 + sizeof(int) + sizeof(int) + 5 + sizeof(int) + 3];
            frame[0] = TransformBatch;
            BinaryPrimitives.WriteInt32LittleEndian(
                frame.AsSpan(1, sizeof(int)),
                2);
            var offset = 1 + sizeof(int);
            BinaryPrimitives.WriteInt32LittleEndian(
                frame.AsSpan(offset, sizeof(int)),
                5);
            offset += sizeof(int);
            "first"u8.CopyTo(frame.AsSpan(offset, 5));
            offset += 5;
            BinaryPrimitives.WriteInt32LittleEndian(
                frame.AsSpan(offset, sizeof(int)),
                10);
            offset += sizeof(int);
            "bad"u8.CopyTo(frame.AsSpan(offset, 3));

            await _input.WriteAsync(frame);
            await _input.FlushAsync();
            _input.Dispose();
            using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(2));
            await _process.WaitForExitAsync(timeout.Token);
            return _process.ExitCode;
        }
        finally
        {
            _gate.Release();
        }
    }

    public async Task DispatchBatchWithoutObservationForExperimentAsync(
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
            frame[0] = TransformBatch;
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
            await _input.WriteAsync(frame);
            await _input.FlushAsync();
        }
        finally
        {
            _gate.Release();
        }
    }

    public async Task<IReadOnlyList<IsolatedWorkerBatchOutcome>> TransformActiveCancellationBatchAsync(
        IReadOnlyList<string> requestIdentities,
        int cancelAtIndex) => await TransformCancellationBatchAsync(
            ActiveCancellationBatch,
            requestIdentities,
            cancelAtIndex,
            TimeSpan.Zero);

    public async Task<IReadOnlyList<IsolatedWorkerBatchOutcome>> TransformNaturalCancellationBatchAsync(
        IReadOnlyList<string> requestIdentities,
        int cancelAtIndex,
        TimeSpan signalDelay) => await TransformCancellationBatchAsync(
            NaturalCancellationBatch,
            requestIdentities,
            cancelAtIndex,
            signalDelay);

    private async Task<IReadOnlyList<IsolatedWorkerBatchOutcome>> TransformCancellationBatchAsync(
        byte operation,
        IReadOnlyList<string> requestIdentities,
        int cancelAtIndex,
        TimeSpan signalDelay)
    {
        ArgumentOutOfRangeException.ThrowIfLessThan(requestIdentities.Count, 1);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(requestIdentities.Count, MaximumBatchMembers);
        ArgumentOutOfRangeException.ThrowIfNegative(cancelAtIndex);
        ArgumentOutOfRangeException.ThrowIfGreaterThanOrEqual(cancelAtIndex, requestIdentities.Count);
        await _gate.WaitAsync();
        try
        {
            ObjectDisposedException.ThrowIf(_disposed, this);
            var encodedLength = checked(1 + sizeof(int) + sizeof(int));
            foreach (var identity in requestIdentities)
            {
                encodedLength = checked(encodedLength + sizeof(int) + Encoding.UTF8.GetByteCount(identity));
            }
            if (encodedLength > MaximumFrameBytes)
            {
                throw new InvalidDataException($"Active cancellation batch exceeds {MaximumFrameBytes} bytes.");
            }
            var frame = new byte[encodedLength];
            frame[0] = operation;
            BinaryPrimitives.WriteInt32LittleEndian(frame.AsSpan(1, sizeof(int)), requestIdentities.Count);
            var offset = 1 + sizeof(int);
            foreach (var identity in requestIdentities)
            {
                var byteCount = Encoding.UTF8.GetByteCount(identity);
                BinaryPrimitives.WriteInt32LittleEndian(frame.AsSpan(offset, sizeof(int)), byteCount);
                offset += sizeof(int);
                offset += Encoding.UTF8.GetBytes(identity, frame.AsSpan(offset, byteCount));
            }
            BinaryPrimitives.WriteInt32LittleEndian(frame.AsSpan(offset, sizeof(int)), cancelAtIndex);
            await _input.WriteAsync(frame);
            await _input.FlushAsync();
            if (await ReadByteAsync() != BatchMemberStarted ||
                checked((int)await ReadUInt32Async()) != cancelAtIndex)
            {
                throw new InvalidDataException("Active batch member start observation changed.");
            }
            var activeIdentity = await ReadStringAsync();
            RequireBatchIdentity(requestIdentities[cancelAtIndex], activeIdentity);
            if (signalDelay > TimeSpan.Zero)
            {
                await Task.Delay(signalDelay);
            }
            await _controlWriter.WriteCancellationAsync(Cancel, activeIdentity);
            return await ReadBatchResponseAsync(requestIdentities);
        }
        finally
        {
            _gate.Release();
        }
    }

    private async Task<IReadOnlyList<IsolatedWorkerBatchOutcome>> SendBatchFrameLockedAsync(
        byte[] frame,
        IReadOnlyList<string> requestIdentities)
    {
        await _input.WriteAsync(frame);
        await _input.FlushAsync();
        return await ReadBatchResponseAsync(requestIdentities);
    }

    private async Task<IReadOnlyList<IsolatedWorkerBatchOutcome>> ReadBatchResponseAsync(
        IReadOnlyList<string> requestIdentities)
    {
        var response = await ReadByteAsync();
        if (response == Error)
        {
            throw await ReadFailureAsync();
        }
        if (response != BatchResult)
        {
            throw new InvalidDataException($"Unexpected batch response: {response}.");
        }
        var count = await ReadUInt32Async();
        if (count != (uint)requestIdentities.Count)
        {
            throw new InvalidDataException(
                $"Worker returned {count} batch outcomes for {requestIdentities.Count} requests.");
        }
        var outcomes = new IsolatedWorkerBatchOutcome[requestIdentities.Count];
        for (var index = 0; index < outcomes.Length; index++)
        {
            var outcomeKind = await ReadByteAsync();
            if (outcomeKind == Result)
            {
                var identity = await ReadStringAsync();
                RequireBatchIdentity(requestIdentities[index], identity);
                outcomes[index] = new IsolatedWorkerBatchOutcome(
                    identity,
                    await ReadStringAsync(),
                    null);
            }
            else if (outcomeKind == Error)
            {
                var failure = await ReadFailureAsync();
                RequireBatchIdentity(requestIdentities[index], failure.RequestId);
                outcomes[index] = new IsolatedWorkerBatchOutcome(
                    failure.RequestId!,
                    null,
                    failure);
            }
            else
            {
                throw new InvalidDataException($"Unexpected batch member response: {outcomeKind}.");
            }
        }
        return outcomes;
    }

    private static void RequireBatchIdentity(string expected, string? actual)
    {
        if (!StringComparer.Ordinal.Equals(expected, actual))
        {
            throw new InvalidDataException("Worker batch response identity did not match its request position.");
        }
    }

    public async Task<string> TransformAsync(
        string requestIdentity,
        CancellationToken cancellationToken)
    {
        if (cancellationToken.IsCancellationRequested)
        {
            await TransformCancelledBeforeDispatchAsync(requestIdentity);
            throw new InvalidOperationException("A cancelled transform unexpectedly completed.");
        }

        var invocation = await StartUnpausedControlledTransformAsync(requestIdentity);
        if (!cancellationToken.CanBeCanceled)
        {
            return await invocation.Completion;
        }

        var cancellationObserved = new TaskCompletionSource(
            TaskCreationOptions.RunContinuationsAsynchronously);
        using var registration = cancellationToken.Register(
            static state => ((TaskCompletionSource)state!).TrySetResult(),
            cancellationObserved);
        if (await Task.WhenAny(invocation.Completion, cancellationObserved.Task) ==
            cancellationObserved.Task)
        {
            await invocation.CancelAsync();
        }
        return await invocation.Completion;
    }

    public async Task TransformCancelledBeforeDispatchAsync(string requestIdentity)
    {
        await _gate.WaitAsync();
        try
        {
            ObjectDisposedException.ThrowIf(_disposed, this);
            await WriteByteAsync(CancelledTransform);
            await WriteStringAsync(requestIdentity);
            await _input.FlushAsync();
            var response = await ReadByteAsync();
            if (response != Error)
            {
                throw new InvalidDataException(
                    $"Cancelled transform unexpectedly returned response: {response}.");
            }
            throw await ReadFailureAsync();
        }
        finally
        {
            _gate.Release();
        }
    }

    public async Task<string> TransformWithXsltInstructionLimitAsync(
        string requestIdentity,
        ulong maximumXsltInstructions)
    {
        await _gate.WaitAsync();
        try
        {
            ObjectDisposedException.ThrowIf(_disposed, this);
            await WriteByteAsync(InstructionLimitedTransform);
            await WriteStringAsync(requestIdentity);
            await WriteUInt64Async(maximumXsltInstructions);
            await _input.FlushAsync();
            var response = await ReadByteAsync();
            if (response == Error)
            {
                throw await ReadFailureAsync();
            }
            if (response != Result)
            {
                throw new InvalidDataException(
                    $"Instruction-limited transform unexpectedly returned: {response}.");
            }
            var correlatedIdentity = await ReadStringAsync();
            if (!StringComparer.Ordinal.Equals(requestIdentity, correlatedIdentity))
            {
                throw new InvalidDataException("Worker response identity did not match the request.");
            }
            return await ReadStringAsync();
        }
        finally
        {
            _gate.Release();
        }
    }

    public async Task<ControlledTransformHandle> StartControlledTransformAsync(
        string requestIdentity) => await StartControlledTransformAsync(
            requestIdentity,
            ControlledTransform);

    public async Task<ControlledTransformHandle> StartUnpausedControlledTransformAsync(
        string requestIdentity) => await StartControlledTransformAsync(
            requestIdentity,
            UnpausedControlledTransform);

    private async Task<ControlledTransformHandle> StartControlledTransformAsync(
        string requestIdentity,
        byte operation)
    {
        await _gate.WaitAsync();
        try
        {
            ObjectDisposedException.ThrowIf(_disposed, this);
            await WriteByteAsync(operation);
            await WriteStringAsync(requestIdentity);
            await _input.FlushAsync();
            var response = await ReadByteAsync();
            if (response != TransformStarted)
            {
                throw new InvalidDataException(
                    $"Controlled transform unexpectedly returned response: {response}.");
            }
            var correlatedIdentity = await ReadStringAsync();
            if (!StringComparer.Ordinal.Equals(requestIdentity, correlatedIdentity))
            {
                throw new InvalidDataException("Controlled transform identity did not match the request.");
            }
            return new ControlledTransformHandle(
                requestIdentity,
                ReadControlledCompletionAsync(requestIdentity),
                SendCancellationAsync);
        }
        catch
        {
            _gate.Release();
            throw;
        }
    }

    public (TimeSpan ProcessorTime, long WorkingSetBytes) ObserveProcess()
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        _process.Refresh();
        return (_process.TotalProcessorTime, _process.WorkingSet64);
    }

    public int ProcessId
    {
        get
        {
            ObjectDisposedException.ThrowIf(_disposed, this);
            return _process.Id;
        }
    }

    public async Task BeginNonCooperatingProbeAsync(string requestIdentity)
    {
        await _gate.WaitAsync();
        try
        {
            ObjectDisposedException.ThrowIf(_disposed, this);
            await WriteByteAsync(NonCooperatingProbe);
            await WriteStringAsync(requestIdentity);
            await _input.FlushAsync();
            var response = await ReadByteAsync();
            if (response != ProbeStarted)
            {
                throw new InvalidDataException($"Unexpected isolation-probe response: {response}.");
            }
            var correlatedIdentity = await ReadStringAsync();
            if (!StringComparer.Ordinal.Equals(requestIdentity, correlatedIdentity))
            {
                throw new InvalidDataException("Isolation-probe identity did not match the request.");
            }
        }
        finally
        {
            _gate.Release();
        }
    }

    public void TerminateForExperiment()
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        if (!_process.HasExited)
        {
            _process.Kill(entireProcessTree: true);
            _process.WaitForExit(2_000);
        }
    }

    public void Dispose()
    {
        if (_disposed)
        {
            return;
        }
        _gate.Wait();
        try
        {
            if (!_process.HasExited)
            {
                try
                {
                    WriteByteAsync(Shutdown).GetAwaiter().GetResult();
                    _input.Flush();
                    if (ReadByteAsync().GetAwaiter().GetResult() != Stopped ||
                        !_process.WaitForExit(2_000))
                    {
                        _process.Kill(entireProcessTree: true);
                    }
                }
                catch (Exception failure) when (failure is IOException or InvalidOperationException)
                {
                    if (!_process.HasExited)
                    {
                        _process.Kill(entireProcessTree: true);
                    }
                }
            }
            _disposed = true;
            _input.Dispose();
            _output.Dispose();
            _process.Dispose();
        }
        finally
        {
            _gate.Release();
            _gate.Dispose();
        }
    }

    private async Task<FastXsltWorkerException> ReadFailureAsync()
    {
        var code = await ReadStringAsync();
        var category = await ReadStringAsync();
        var requestId = NullIfEmpty(await ReadStringAsync());
        var resource = NullIfEmpty(await ReadStringAsync());
        var start = NullIfEmpty(await ReadStringAsync());
        var end = NullIfEmpty(await ReadStringAsync());
        var location = resource is null
            ? null
            : new FastXsltDiagnosticLocation(
                resource,
                ulong.Parse(start ?? throw new InvalidDataException("Missing location start.")),
                ulong.Parse(end ?? throw new InvalidDataException("Missing location end.")));
        return new FastXsltWorkerException(
            code,
            category,
            requestId,
            location,
            await ReadStringAsync());
    }

    private async Task<string> ReadControlledCompletionAsync(string requestIdentity)
    {
        try
        {
            var response = await ReadByteAsync();
            if (response == Error)
            {
                throw await ReadFailureAsync();
            }
            if (response != Result)
            {
                throw new InvalidDataException(
                    $"Unexpected controlled transform response: {response}.");
            }
            var correlatedIdentity = await ReadStringAsync();
            if (!StringComparer.Ordinal.Equals(requestIdentity, correlatedIdentity))
            {
                throw new InvalidDataException("Worker response identity did not match the request.");
            }
            return await ReadStringAsync();
        }
        finally
        {
            _gate.Release();
        }
    }

    private async Task SendCancellationAsync(string requestIdentity)
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        await _controlWriter.WriteCancellationAsync(Cancel, requestIdentity);
    }

    private async Task WriteByteAsync(byte value) =>
        await _input.WriteAsync(new[] { value });

    private async Task WriteStringAsync(string value) =>
        await WriteBytesAsync(Encoding.UTF8.GetBytes(value));

    private async Task WriteUInt64Async(ulong value)
    {
        var bytes = new byte[8];
        BinaryPrimitives.WriteUInt64LittleEndian(bytes, value);
        await _input.WriteAsync(bytes);
    }

    private async Task WriteBytesAsync(byte[] value)
    {
        if (value.Length > MaximumFrameBytes)
        {
            throw new InvalidDataException($"Frame exceeds {MaximumFrameBytes} bytes.");
        }
        var length = new byte[4];
        BinaryPrimitives.WriteInt32LittleEndian(length, value.Length);
        await _input.WriteAsync(length);
        await _input.WriteAsync(value);
    }

    private async Task<byte> ReadByteAsync()
    {
        var value = new byte[1];
        await _output.ReadExactlyAsync(value);
        return value[0];
    }

    private async Task<ulong> ReadUInt64Async()
    {
        var value = new byte[sizeof(ulong)];
        await _output.ReadExactlyAsync(value);
        return BinaryPrimitives.ReadUInt64LittleEndian(value);
    }

    private async Task<uint> ReadUInt32Async()
    {
        var value = new byte[sizeof(uint)];
        await _output.ReadExactlyAsync(value);
        return BinaryPrimitives.ReadUInt32LittleEndian(value);
    }

    private async Task<string> ReadStringAsync()
    {
        var lengthBytes = new byte[4];
        await _output.ReadExactlyAsync(lengthBytes);
        var length = BinaryPrimitives.ReadInt32LittleEndian(lengthBytes);
        if (length < 0 || length > MaximumFrameBytes)
        {
            throw new InvalidDataException($"Invalid worker frame length: {length}.");
        }
        var value = new byte[length];
        await _output.ReadExactlyAsync(value);
        return Encoding.UTF8.GetString(value);
    }

    private static string? NullIfEmpty(string value) => value.Length == 0 ? null : value;
}

public sealed record IsolatedWorkerMeasuredTransform(
    string Result,
    IsolatedWorkerTransformTiming Timing);

public sealed record IsolatedWorkerTransformTiming(
    double GateMicroseconds,
    double RequestWriteMicroseconds,
    double RequestFlushMicroseconds,
    double ResponseWaitAndReadMicroseconds,
    double WorkerDecodeMicroseconds,
    double WorkerQueueMicroseconds,
    double WorkerExecutionMicroseconds,
    double InstrumentedTotalMicroseconds)
{
    public double UnattributedRoundTripMicroseconds => Math.Max(
        0,
        InstrumentedTotalMicroseconds
            - GateMicroseconds
            - RequestWriteMicroseconds
            - RequestFlushMicroseconds
            - WorkerDecodeMicroseconds
            - WorkerQueueMicroseconds
            - WorkerExecutionMicroseconds);
}

public sealed record IsolatedWorkerBatchOutcome(
    string RequestIdentity,
    string? Result,
    FastXsltWorkerException? Failure);

public sealed record IsolatedWorkerBatchRequest(
    string RequestIdentity,
    bool Cancelled = false,
    ulong? MaximumXsltInstructions = null);

public sealed record IsolatedBatchLossObservation(
    int MemberIndex,
    string RequestIdentity,
    string Phase);

public sealed record IsolatedBatchTransferLossProbe(
    IReadOnlyList<IsolatedWorkerBatchOutcome> CompleteOutcomes,
    int PartialMemberIndex,
    string PartialRequestIdentity,
    int DeclaredResultBytes,
    int ObservedResultBytes);

internal sealed class SerializedWorkerControlWriter(Stream output, int maximumFrameBytes)
{
    private readonly SemaphoreSlim _gate = new(1, 1);

    public async Task WriteCancellationAsync(byte operation, string requestIdentity)
    {
        var identity = Encoding.UTF8.GetBytes(requestIdentity);
        if (identity.Length > maximumFrameBytes)
        {
            throw new InvalidDataException($"Frame exceeds {maximumFrameBytes} bytes.");
        }
        var frame = new byte[1 + sizeof(int) + identity.Length];
        frame[0] = operation;
        BinaryPrimitives.WriteInt32LittleEndian(frame.AsSpan(1, sizeof(int)), identity.Length);
        identity.CopyTo(frame.AsSpan(1 + sizeof(int)));

        await _gate.WaitAsync();
        try
        {
            await output.WriteAsync(frame);
            await output.FlushAsync();
        }
        finally
        {
            _gate.Release();
        }
    }
}

public sealed class ControlledTransformHandle(
    string requestIdentity,
    Task<string> completion,
    Func<string, Task> sendCancellation)
{
    private int _cancellationSent;

    public string RequestIdentity { get; } = requestIdentity;
    public Task<string> Completion { get; } = completion;

    public Task CancelAsync() => Interlocked.Exchange(ref _cancellationSent, 1) == 0
        ? sendCancellation(RequestIdentity)
        : Task.CompletedTask;

    public Task SendUnrelatedCancellationForExperimentAsync(string unrelatedRequestIdentity) =>
        sendCancellation(unrelatedRequestIdentity);
}

public sealed class FastXsltWorkerException(
    string code,
    string category,
    string? requestId,
    FastXsltDiagnosticLocation? location,
    string detail) : Exception(detail)
{
    public string Code { get; } = code;
    public string Category { get; } = category;
    public string? RequestId { get; } = requestId;
    public FastXsltDiagnosticLocation? Location { get; } = location;
    public string Detail { get; } = detail;
}

public sealed record FastXsltDiagnosticLocation(string Resource, ulong Start, ulong End);
