using System.Diagnostics;
using System.Text;

public sealed partial class FastXsltWorkerClient
{
    // Same initialize opcode, framing, limits, readiness and error decoding as
    // StartAsync. No new protocol operation or retry is introduced.
    internal static async Task<(FastXsltWorkerClient Client, IsolatedCreationTiming Timing)>
        StartMeasuredAsync(string workerPath, string sourceIdentity, byte[] source,
            string stylesheetIdentity, byte[] stylesheet)
    {
        if (!File.Exists(workerPath))
            throw new FileNotFoundException("Build the isolated worker first.", workerPath);
        var total = Stopwatch.GetTimestamp();
        var phase = total;
        var sourceId = Encoding.UTF8.GetBytes(sourceIdentity);
        var styleId = Encoding.UTF8.GetBytes(stylesheetIdentity);
        var encoding = Stopwatch.GetElapsedTime(phase).TotalMicroseconds;
        phase = Stopwatch.GetTimestamp();
        var process = Process.Start(new ProcessStartInfo(workerPath)
        {
            RedirectStandardInput = true, RedirectStandardOutput = true,
            RedirectStandardError = true, UseShellExecute = false, CreateNoWindow = true,
        }) ?? throw new InvalidOperationException("Failed to start the worker.");
        var launch = Stopwatch.GetElapsedTime(phase).TotalMicroseconds;
        var client = new FastXsltWorkerClient(process);
        try
        {
            phase = Stopwatch.GetTimestamp();
            await client.WriteByteAsync(Initialize);
            await client.WriteBytesAsync(sourceId);
            await client.WriteBytesAsync(source);
            await client.WriteBytesAsync(styleId);
            await client.WriteBytesAsync(stylesheet);
            var write = Stopwatch.GetElapsedTime(phase).TotalMicroseconds;
            phase = Stopwatch.GetTimestamp();
            await client._input.FlushAsync();
            var flush = Stopwatch.GetElapsedTime(phase).TotalMicroseconds;
            phase = Stopwatch.GetTimestamp();
            var response = await client.ReadByteAsync();
            if (response == Error) throw await client.ReadFailureAsync();
            if (response != Ready)
                throw new InvalidDataException($"Unexpected initialization response: {response}.");
            var ready = Stopwatch.GetElapsedTime(phase).TotalMicroseconds;
            return (client, new IsolatedCreationTiming(encoding, launch, write, flush, ready,
                Stopwatch.GetElapsedTime(total).TotalMicroseconds, process.Id,
                checked(1L + 4 * sizeof(int) + sourceId.Length + styleId.Length + source.Length + stylesheet.Length)));
        }
        catch
        {
            client.Dispose();
            throw;
        }
    }
}

internal sealed record IsolatedCreationTiming(
    double IdentityEncodingMicroseconds, double ProcessLaunchReturnMicroseconds,
    double RequestWriteMicroseconds, double RequestFlushMicroseconds,
    double ReadinessWaitMicroseconds, double TotalCreationMicroseconds,
    int ProcessId, long EncodedInitializationBytes);
