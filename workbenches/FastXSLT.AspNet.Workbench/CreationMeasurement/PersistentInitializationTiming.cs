using System.Diagnostics;
using System.Text;

public sealed partial class FastXsltWorkerClient
{
    // Private sequential measurement over the existing Initialize command.
    // No new protocol or supported generation replacement API is selected.
    internal async Task<PersistentInitializationTiming> ReinitializeMeasuredAsync(
        string sourceIdentity, byte[] source, string stylesheetIdentity, byte[] stylesheet)
    {
        await _gate.WaitAsync();
        var retire = false;
        try
        {
            ObjectDisposedException.ThrowIf(_disposed, this);
            var total = Stopwatch.GetTimestamp();
            var phase = total;
            var sourceId = Encoding.UTF8.GetBytes(sourceIdentity);
            var styleId = Encoding.UTF8.GetBytes(stylesheetIdentity);
            var encoding = Stopwatch.GetElapsedTime(phase).TotalMicroseconds;
            // Validate before emitting the opcode: an over-limit field must not
            // leave a partial command on a worker intended for continued use.
            if (sourceId.Length > MaximumFrameBytes || styleId.Length > MaximumFrameBytes ||
                source.Length > MaximumFrameBytes || stylesheet.Length > MaximumFrameBytes)
                throw new ArgumentOutOfRangeException(nameof(source), "Initialization field exceeds frame limit.");
            phase = Stopwatch.GetTimestamp();
            await WriteByteAsync(Initialize);
            await WriteBytesAsync(sourceId);
            await WriteBytesAsync(source);
            await WriteBytesAsync(styleId);
            await WriteBytesAsync(stylesheet);
            var write = Stopwatch.GetElapsedTime(phase).TotalMicroseconds;
            phase = Stopwatch.GetTimestamp();
            await _input.FlushAsync();
            var flush = Stopwatch.GetElapsedTime(phase).TotalMicroseconds;
            phase = Stopwatch.GetTimestamp();
            var response = await ReadByteAsync();
            if (response == Error) throw await ReadFailureAsync();
            if (response != Ready) throw new InvalidDataException("Unexpected initialization response.");
            return new(encoding, write, flush, Stopwatch.GetElapsedTime(phase).TotalMicroseconds,
                Stopwatch.GetElapsedTime(total).TotalMicroseconds);
        }
        catch (FastXsltWorkerException) { throw; } // Fully decoded semantic failure; old engine remains.
        catch (ArgumentOutOfRangeException) { throw; } // No frame was emitted.
        catch { retire = true; throw; } // A partial/unknown transaction is not reusable.
        finally
        {
            _gate.Release();
            if (retire)
            {
                TerminateForExperiment(); // Never attempt graceful framing on a partial transaction.
                Dispose(); // Disposal acquires the gate; never do it while holding it.
            }
        }
    }
}

internal sealed record PersistentInitializationTiming(double IdentityEncodingUs,
    double WriteUs, double FlushUs, double ReadinessUs, double TotalUs);
