using System.Buffers.Binary;
using System.Diagnostics;
using System.Text;

public sealed class FastXsltWorkerClient : IDisposable
{
    private const byte Initialize = 1;
    private const byte Transform = 2;
    private const byte Shutdown = 3;
    private const byte Ready = 0x81;
    private const byte Result = 0x82;
    private const byte Stopped = 0x83;
    private const byte Error = 0xff;
    private const int MaximumFrameBytes = 1_048_576;

    private readonly Process _process;
    private readonly Stream _input;
    private readonly Stream _output;
    private readonly SemaphoreSlim _gate = new(1, 1);
    private bool _disposed;

    private FastXsltWorkerClient(Process process)
    {
        _process = process;
        _input = process.StandardInput.BaseStream;
        _output = process.StandardOutput.BaseStream;
        process.BeginErrorReadLine();
    }

    public static async Task<FastXsltWorkerClient> StartAsync(
        string workerPath,
        string sourceIdentity,
        byte[] source,
        string stylesheetIdentity,
        byte[] stylesheet)
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
            await client.WriteByteAsync(Initialize);
            await client.WriteStringAsync(sourceIdentity);
            await client.WriteBytesAsync(source);
            await client.WriteStringAsync(stylesheetIdentity);
            await client.WriteBytesAsync(stylesheet);
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
                WriteByteAsync(Shutdown).GetAwaiter().GetResult();
                _input.Flush();
                if (ReadByteAsync().GetAwaiter().GetResult() != Stopped ||
                    !_process.WaitForExit(2_000))
                {
                    _process.Kill(entireProcessTree: true);
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

    private async Task<FastXsltWorkerException> ReadFailureAsync() => new(
        await ReadStringAsync(),
        await ReadStringAsync(),
        NullIfEmpty(await ReadStringAsync()),
        await ReadStringAsync());

    private async Task WriteByteAsync(byte value) =>
        await _input.WriteAsync(new[] { value });

    private async Task WriteStringAsync(string value) =>
        await WriteBytesAsync(Encoding.UTF8.GetBytes(value));

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

public sealed class FastXsltWorkerException(
    string code,
    string category,
    string? requestId,
    string detail) : Exception(detail)
{
    public string Code { get; } = code;
    public string Category { get; } = category;
    public string? RequestId { get; } = requestId;
    public string Detail { get; } = detail;
}
