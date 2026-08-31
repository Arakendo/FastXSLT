using System.Buffers.Binary;
using System.Diagnostics;
using System.Text;

public sealed class FastXsltWorkerClient : IDisposable
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
    private const byte Ready = 0x81;
    private const byte Result = 0x82;
    private const byte Stopped = 0x83;
    private const byte ProbeStarted = 0x84;
    private const byte TransformStarted = 0x85;
    private const byte Error = 0xff;
    private const int MaximumFrameBytes = 1_048_576;

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
