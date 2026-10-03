using System.Diagnostics;
using System.Text;

public sealed partial class NativeFastXsltClient
{
    // This opt-in probe calls exactly the existing export and ownership decoder.
    // It introduces no native observation export or foreign-memory lifetime.
    internal static (NativeFastXsltClient Client, NativeCreationTiming Timing) CreateMeasured(
        string sourceIdentity, byte[] source, string stylesheetIdentity, byte[] stylesheet)
    {
        var allocated = GC.GetAllocatedBytesForCurrentThread();
        var total = Stopwatch.GetTimestamp();
        var phase = total;
        AssertAbiVersion();
        var version = Stopwatch.GetElapsedTime(phase).TotalMicroseconds;
        phase = Stopwatch.GetTimestamp();
        var sourceId = Encoding.UTF8.GetBytes(sourceIdentity);
        var styleId = Encoding.UTF8.GetBytes(stylesheetIdentity);
        var encoding = Stopwatch.GetElapsedTime(phase).TotalMicroseconds;
        phase = Stopwatch.GetTimestamp();
        var outcome = NativeMethods.Create(sourceId, (nuint)sourceId.Length,
            source, (nuint)source.Length, styleId, (nuint)styleId.Length,
            stylesheet, (nuint)stylesheet.Length);
        var creation = Stopwatch.GetElapsedTime(phase).TotalMicroseconds;
        phase = Stopwatch.GetTimestamp();
        var client = FromCreationOutcome(outcome);
        var ownership = Stopwatch.GetElapsedTime(phase).TotalMicroseconds;
        return (client, new NativeCreationTiming(version, encoding, creation, ownership,
            Stopwatch.GetElapsedTime(total).TotalMicroseconds,
            GC.GetAllocatedBytesForCurrentThread() - allocated,
            checked((long)sourceId.Length + styleId.Length + source.Length + stylesheet.Length)));
    }
}

internal sealed record NativeCreationTiming(
    double AbiCheckMicroseconds, double IdentityEncodingMicroseconds,
    double CombinedNativeCreateMicroseconds, double OutcomeOwnershipMicroseconds,
    double TotalCreationMicroseconds, long ManagedAllocatedBytes, long SuppliedCopyBytes);
