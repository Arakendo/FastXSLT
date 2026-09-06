using System.Diagnostics;
using System.Text;

public static class IsolatedBoundaryBreakdown
{
    private static readonly BoundaryTier[] Tiers =
    [
        new("items-5", 5, 20),
        new("items-50", 50, 4),
        new("items-500", 500, 1)
    ];

    public static async Task<IsolatedBoundaryBreakdownReport> RunAsync(
        string workerPath,
        byte[] stylesheet,
        int baseRequests)
    {
        baseRequests = Math.Clamp(baseRequests, 1, 10_000);
        var measurements = new List<IsolatedBoundaryTierMeasurement>(Tiers.Length);

        foreach (var tier in Tiers)
        {
            var source = BuildSource(tier.Items);
            var requests = Math.Min(10_000, checked(baseRequests * tier.RequestMultiplier));
            var expected = $"<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>{tier.Items}.00</out>";
            using var worker = await FastXsltWorkerClient.StartAsync(
                workerPath,
                $"urn:fastxslt:isolated-boundary:{tier.Name}:source",
                source,
                $"urn:fastxslt:isolated-boundary:{tier.Name}:stylesheet",
                stylesheet);

            RequireExpected(
                await worker.TransformAsync($"{tier.Name}-ordinary-warm"),
                expected);
            RequireExpected(
                (await worker.TransformMeasuredAsync($"{tier.Name}-measured-warm")).Result,
                expected);

            var ordinary = await MeasureOrdinaryAsync(worker, tier.Name, expected, requests);
            var phases = await MeasurePhasesAsync(worker, tier.Name, expected, requests);
            measurements.Add(new IsolatedBoundaryTierMeasurement(
                tier.Name,
                tier.Items,
                source.Length,
                Encoding.UTF8.GetByteCount(expected),
                requests,
                ordinary,
                phases));
        }

        return new IsolatedBoundaryBreakdownReport(
            baseRequests,
            Environment.Version.ToString(),
            Stopwatch.Frequency,
            measurements,
            "Worker values are elapsed durations measured on the worker clock; managed values are elapsed durations measured on the host clock. No timestamps are subtracted across processes. Unattributed round-trip time is a difference of independently measured durations and includes response framing/write/flush, pipe transit and wakeups, managed reads, and probe overhead.");
    }

    private static async Task<IsolatedBoundaryWholeMeasurement> MeasureOrdinaryAsync(
        FastXsltWorkerClient worker,
        string tier,
        string expected,
        int requests)
    {
        Collect();
        var allocatedBefore = GC.GetTotalAllocatedBytes(precise: true);
        var started = Stopwatch.StartNew();
        for (var index = 0; index < requests; index++)
        {
            RequireExpected(
                await worker.TransformAsync($"{tier}-ordinary-{index}"),
                expected);
        }
        started.Stop();
        return new IsolatedBoundaryWholeMeasurement(
            started.Elapsed.TotalMilliseconds,
            requests / started.Elapsed.TotalSeconds,
            started.Elapsed.TotalMicroseconds / requests,
            (GC.GetTotalAllocatedBytes(precise: true) - allocatedBefore) / (double)requests);
    }

    private static async Task<IsolatedBoundaryPhaseMeans> MeasurePhasesAsync(
        FastXsltWorkerClient worker,
        string tier,
        string expected,
        int requests)
    {
        var totals = new double[9];
        for (var index = 0; index < requests; index++)
        {
            var sample = await worker.TransformMeasuredAsync($"{tier}-phases-{index}");
            RequireExpected(sample.Result, expected);
            var timing = sample.Timing;
            totals[0] += timing.GateMicroseconds;
            totals[1] += timing.RequestWriteMicroseconds;
            totals[2] += timing.RequestFlushMicroseconds;
            totals[3] += timing.ResponseWaitAndReadMicroseconds;
            totals[4] += timing.WorkerDecodeMicroseconds;
            totals[5] += timing.WorkerQueueMicroseconds;
            totals[6] += timing.WorkerExecutionMicroseconds;
            totals[7] += timing.UnattributedRoundTripMicroseconds;
            totals[8] += timing.InstrumentedTotalMicroseconds;
        }
        return new IsolatedBoundaryPhaseMeans(
            totals[0] / requests,
            totals[1] / requests,
            totals[2] / requests,
            totals[3] / requests,
            totals[4] / requests,
            totals[5] / requests,
            totals[6] / requests,
            totals[7] / requests,
            totals[8] / requests);
    }

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
                $"Isolated boundary benchmark returned {actual}; expected {expected}.");
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

public sealed record IsolatedBoundaryBreakdownReport(
    int BaseRequestsAtLargestTier,
    string RuntimeVersion,
    long StopwatchFrequency,
    IReadOnlyList<IsolatedBoundaryTierMeasurement> Measurements,
    string InterpretationConstraint);

public sealed record IsolatedBoundaryTierMeasurement(
    string Tier,
    int Items,
    int SourceBytes,
    int ResultBytes,
    int Requests,
    IsolatedBoundaryWholeMeasurement Ordinary,
    IsolatedBoundaryPhaseMeans InstrumentedMeans);

public sealed record IsolatedBoundaryWholeMeasurement(
    double ElapsedMilliseconds,
    double TransformsPerSecond,
    double MeanMicroseconds,
    double ManagedAllocatedBytesPerRequest);

public sealed record IsolatedBoundaryPhaseMeans(
    double GateMicroseconds,
    double RequestWriteMicroseconds,
    double RequestFlushMicroseconds,
    double ResponseWaitAndReadMicroseconds,
    double WorkerDecodeMicroseconds,
    double WorkerQueueMicroseconds,
    double WorkerExecutionMicroseconds,
    double UnattributedRoundTripMicroseconds,
    double InstrumentedTotalMicroseconds);

