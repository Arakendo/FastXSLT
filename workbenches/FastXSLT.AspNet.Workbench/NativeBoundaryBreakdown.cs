using System.Diagnostics;
using System.Text;

public static class NativeBoundaryBreakdown
{
    private static readonly BoundaryTier[] Tiers =
    [
        new("items-5", 5, 20),
        new("items-50", 50, 4),
        new("items-500", 500, 1)
    ];

    public static async Task<NativeBoundaryBreakdownReport> RunAsync(
        byte[] stylesheet,
        int baseRequests)
    {
        baseRequests = Math.Clamp(baseRequests, 1, 10_000);
        var measurements = new List<NativeBoundaryTierMeasurement>(Tiers.Length);

        foreach (var tier in Tiers)
        {
            var source = BuildSource(tier.Items);
            var requests = Math.Min(10_000, checked(baseRequests * tier.RequestMultiplier));
            var expected = $"<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>{tier.Items}.00</out>";
            using var direct = NativeFastXsltClient.Create(
                $"urn:fastxslt:native-boundary-breakdown:{tier.Name}:direct:source",
                source,
                $"urn:fastxslt:native-boundary-breakdown:{tier.Name}:direct:stylesheet",
                stylesheet);
            using var pool = NativeFastXsltPool.Create(
                $"urn:fastxslt:native-boundary-breakdown:{tier.Name}:pool:source",
                source,
                $"urn:fastxslt:native-boundary-breakdown:{tier.Name}:pool:stylesheet",
                stylesheet,
                engines: 1);

            RequireExpected(direct.Transform($"{tier.Name}-direct-warm"), expected);
            RequireExpected(
                await pool.TransformAsync($"{tier.Name}-pool-warm"),
                expected);
            RequireExpected(
                direct.TransformMeasured($"{tier.Name}-instrumented-warm").Result,
                expected);

            var directWhole = MeasureDirect(direct, tier.Name, expected, requests);
            var poolWhole = await MeasurePoolAsync(pool, tier.Name, expected, requests);
            var phases = MeasurePhases(direct, tier.Name, expected, requests);
            measurements.Add(new NativeBoundaryTierMeasurement(
                tier.Name,
                tier.Items,
                source.Length,
                Encoding.UTF8.GetByteCount(expected),
                requests,
                directWhole,
                poolWhole,
                poolWhole.MeanMicroseconds - directWhole.MeanMicroseconds,
                phases));
        }

        return new NativeBoundaryBreakdownReport(
            baseRequests,
            Environment.Version.ToString(),
            Stopwatch.Frequency,
            measurements,
            "Phase timings use Stopwatch around each existing managed/ABI step and therefore include probe overhead. TransformExport includes Rust execution and registry publication; no native export or engine semantic was changed.");
    }

    private static NativeBoundaryWholeMeasurement MeasureDirect(
        NativeFastXsltClient client,
        string tier,
        string expected,
        int requests)
    {
        Collect();
        var allocatedBefore = GC.GetTotalAllocatedBytes(precise: true);
        var started = Stopwatch.StartNew();
        for (var index = 0; index < requests; index++)
        {
            RequireExpected(client.Transform($"{tier}-direct-{index}"), expected);
        }
        started.Stop();
        return WholeMeasurement(started.Elapsed, requests, allocatedBefore);
    }

    private static async Task<NativeBoundaryWholeMeasurement> MeasurePoolAsync(
        NativeFastXsltPool pool,
        string tier,
        string expected,
        int requests)
    {
        Collect();
        var allocatedBefore = GC.GetTotalAllocatedBytes(precise: true);
        var started = Stopwatch.StartNew();
        for (var index = 0; index < requests; index++)
        {
            RequireExpected(await pool.TransformAsync($"{tier}-pool-{index}"), expected);
        }
        started.Stop();
        return WholeMeasurement(started.Elapsed, requests, allocatedBefore);
    }

    private static NativeBoundaryPhaseMeans MeasurePhases(
        NativeFastXsltClient client,
        string tier,
        string expected,
        int requests)
    {
        var totals = new double[10];
        for (var index = 0; index < requests; index++)
        {
            var sample = client.TransformMeasured($"{tier}-phases-{index}");
            RequireExpected(sample.Result, expected);
            var timing = sample.Timing;
            totals[0] += timing.GateMicroseconds;
            totals[1] += timing.RequestEncodingMicroseconds;
            totals[2] += timing.TransformExportMicroseconds;
            totals[3] += timing.OutcomeKindMicroseconds;
            totals[4] += timing.OutcomeLengthMicroseconds;
            totals[5] += timing.BufferAllocationMicroseconds;
            totals[6] += timing.OutcomeCopyMicroseconds;
            totals[7] += timing.ResultDecodingMicroseconds;
            totals[8] += timing.OutcomeReleaseMicroseconds;
            totals[9] += timing.InstrumentedTotalMicroseconds;
        }
        return new NativeBoundaryPhaseMeans(
            totals[0] / requests,
            totals[1] / requests,
            totals[2] / requests,
            totals[3] / requests,
            totals[4] / requests,
            totals[5] / requests,
            totals[6] / requests,
            totals[7] / requests,
            totals[8] / requests,
            totals[9] / requests);
    }

    private static NativeBoundaryWholeMeasurement WholeMeasurement(
        TimeSpan elapsed,
        int requests,
        long allocatedBefore) => new(
            elapsed.TotalMilliseconds,
            requests / elapsed.TotalSeconds,
            elapsed.TotalMicroseconds / requests,
            (GC.GetTotalAllocatedBytes(precise: true) - allocatedBefore) / (double)requests);

    private static void Collect()
    {
        GC.Collect(GC.MaxGeneration, GCCollectionMode.Forced, blocking: true, compacting: false);
        GC.WaitForPendingFinalizers();
        GC.Collect(GC.MaxGeneration, GCCollectionMode.Forced, blocking: true, compacting: false);
    }

    private static void RequireExpected(string actual, string expected)
    {
        if (!StringComparer.Ordinal.Equals(actual, expected))
        {
            throw new InvalidOperationException(
                $"Native boundary benchmark returned {actual}; expected {expected}.");
        }
    }

    private static byte[] BuildSource(int items)
    {
        var source = new StringBuilder("<?xml version=\"1.0\"?><order>");
        for (var index = 0; index < items; index++)
        {
            source.Append("<order-item price=\"1.00\" qty=\"1\"/>");
        }
        source.Append("</order>");
        return Encoding.UTF8.GetBytes(source.ToString());
    }

    private sealed record BoundaryTier(string Name, int Items, int RequestMultiplier);
}

public sealed record NativeBoundaryBreakdownReport(
    int BaseRequestsAtLargestTier,
    string RuntimeVersion,
    long StopwatchFrequency,
    IReadOnlyList<NativeBoundaryTierMeasurement> Measurements,
    string InterpretationConstraint);

public sealed record NativeBoundaryTierMeasurement(
    string Tier,
    int Items,
    int SourceBytes,
    int ResultBytes,
    int Requests,
    NativeBoundaryWholeMeasurement Direct,
    NativeBoundaryWholeMeasurement OneSlotPool,
    double ObservedPoolOverheadMicroseconds,
    NativeBoundaryPhaseMeans InstrumentedDirectMeans);

public sealed record NativeBoundaryWholeMeasurement(
    double ElapsedMilliseconds,
    double TransformsPerSecond,
    double MeanMicroseconds,
    double ManagedAllocatedBytesPerRequest);

public sealed record NativeBoundaryPhaseMeans(
    double GateMicroseconds,
    double RequestEncodingMicroseconds,
    double TransformExportMicroseconds,
    double OutcomeKindMicroseconds,
    double OutcomeLengthMicroseconds,
    double BufferAllocationMicroseconds,
    double OutcomeCopyMicroseconds,
    double ResultDecodingMicroseconds,
    double OutcomeReleaseMicroseconds,
    double InstrumentedTotalMicroseconds);
