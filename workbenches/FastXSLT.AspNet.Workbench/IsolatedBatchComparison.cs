using System.Diagnostics;
using System.Text;

public static class IsolatedBatchComparison
{
    private static readonly BoundaryTier[] Tiers =
    [
        new("items-5", 5, 20),
        new("items-50", 50, 4),
        new("items-500", 500, 1)
    ];

    private static readonly int[] BatchSizes = [1, 8, 32, 128];

    public static async Task<IsolatedBatchComparisonReport> RunAsync(
        string workerPath,
        byte[] stylesheet,
        int baseRequests,
        int orderOffset)
    {
        baseRequests = Math.Clamp(baseRequests, 1, 10_000);
        var modes = new[] { 0, 1, 8, 32, 128 };
        orderOffset = Math.Abs(orderOffset % modes.Length);
        var measurements = new List<IsolatedBatchMeasurement>();

        foreach (var tier in Tiers)
        {
            var source = BuildSource(tier.Items);
            var requests = Math.Min(10_000, checked(baseRequests * tier.RequestMultiplier));
            var expected = $"<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>{tier.Items}.00</out>";
            using var worker = await FastXsltWorkerClient.StartAsync(
                workerPath,
                $"urn:fastxslt:isolated-batch:{tier.Name}:source",
                source,
                $"urn:fastxslt:isolated-batch:{tier.Name}:stylesheet",
                stylesheet);

            RequireExpected(
                await worker.TransformAsync($"{tier.Name}-ordinary-warm"),
                expected);
            foreach (var batchSize in BatchSizes)
            {
                var warm = await worker.TransformBatchAsync(
                    BuildIdentities(tier.Name, batchSize, "warm"));
                RequireOutcomes(warm, expected);
            }

            for (var modeIndex = 0; modeIndex < modes.Length; modeIndex++)
            {
                var batchSize = modes[(modeIndex + orderOffset) % modes.Length];
                measurements.Add(batchSize == 0
                    ? await MeasureOrdinaryAsync(
                        worker,
                        tier.Name,
                        tier.Items,
                        source.Length,
                        expected,
                        requests)
                    : await MeasureBatchAsync(
                        worker,
                        tier.Name,
                        tier.Items,
                        source.Length,
                        expected,
                        requests,
                        batchSize));
            }
        }

        return new IsolatedBatchComparisonReport(
            baseRequests,
            Environment.Version.ToString(),
            Stopwatch.Frequency,
            orderOffset,
            measurements,
            "The worker retains one sequential execution lane. Batch latency is the aggregate-response transaction latency observed by every member in that frame; it is not divided by member count. The order offset rotates ordinary and batch-size lanes across repeated endpoint calls.");
    }

    private static async Task<IsolatedBatchMeasurement> MeasureOrdinaryAsync(
        FastXsltWorkerClient worker,
        string tier,
        int items,
        int sourceBytes,
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
        return Measurement(
            tier,
            items,
            sourceBytes,
            expected,
            "ordinary",
            1,
            requests,
            requests,
            started.Elapsed,
            allocatedBefore,
            []);
    }

    private static async Task<IsolatedBatchMeasurement> MeasureBatchAsync(
        FastXsltWorkerClient worker,
        string tier,
        int items,
        int sourceBytes,
        string expected,
        int requests,
        int batchSize)
    {
        var batches = new List<string[]>((requests + batchSize - 1) / batchSize);
        var remaining = requests;
        var batchIndex = 0;
        while (remaining > 0)
        {
            var members = Math.Min(batchSize, remaining);
            batches.Add(BuildIdentities(tier, members, $"batch-{batchSize}-{batchIndex}"));
            remaining -= members;
            batchIndex++;
        }

        Collect();
        var allocatedBefore = GC.GetTotalAllocatedBytes(precise: true);
        var transactionLatencies = new double[batches.Count];
        var started = Stopwatch.StartNew();
        for (var index = 0; index < batches.Count; index++)
        {
            var transactionStarted = Stopwatch.GetTimestamp();
            var outcomes = await worker.TransformBatchAsync(batches[index]);
            transactionLatencies[index] = Stopwatch.GetElapsedTime(transactionStarted).TotalMicroseconds;
            RequireOutcomes(outcomes, expected);
        }
        started.Stop();
        Array.Sort(transactionLatencies);
        return Measurement(
            tier,
            items,
            sourceBytes,
            expected,
            "bounded-batch",
            batchSize,
            requests,
            batches.Count,
            started.Elapsed,
            allocatedBefore,
            transactionLatencies);
    }

    private static IsolatedBatchMeasurement Measurement(
        string tier,
        int items,
        int sourceBytes,
        string expected,
        string mode,
        int batchSize,
        int requests,
        int transactions,
        TimeSpan elapsed,
        long allocatedBefore,
        IReadOnlyList<double> transactionLatencies) => new(
            tier,
            items,
            sourceBytes,
            Encoding.UTF8.GetByteCount(expected),
            mode,
            batchSize,
            requests,
            transactions,
            elapsed.TotalMilliseconds,
            requests / elapsed.TotalSeconds,
            elapsed.TotalMicroseconds / requests,
            (GC.GetTotalAllocatedBytes(precise: true) - allocatedBefore) / (double)requests,
            Percentile(transactionLatencies, 0.50),
            Percentile(transactionLatencies, 0.95),
            Percentile(transactionLatencies, 0.99));

    private static double? Percentile(IReadOnlyList<double> sorted, double percentile)
    {
        if (sorted.Count == 0)
        {
            return null;
        }
        var index = (int)Math.Ceiling(percentile * sorted.Count) - 1;
        return sorted[Math.Clamp(index, 0, sorted.Count - 1)];
    }

    private static string[] BuildIdentities(string tier, int count, string prefix)
    {
        var identities = new string[count];
        for (var index = 0; index < count; index++)
        {
            identities[index] = $"{tier}-{prefix}-{index}";
        }
        return identities;
    }

    private static void RequireOutcomes(
        IReadOnlyList<IsolatedWorkerBatchOutcome> outcomes,
        string expected)
    {
        foreach (var outcome in outcomes)
        {
            if (outcome.Failure is not null)
            {
                throw outcome.Failure;
            }
            RequireExpected(outcome.Result, expected);
        }
    }

    private static void RequireExpected(string? actual, string expected)
    {
        if (!StringComparer.Ordinal.Equals(actual, expected))
        {
            throw new InvalidOperationException(
                $"Isolated batch benchmark returned {actual}; expected {expected}.");
        }
    }

    private static void Collect()
    {
        GC.Collect(GC.MaxGeneration, GCCollectionMode.Forced, blocking: true, compacting: false);
        GC.WaitForPendingFinalizers();
        GC.Collect(GC.MaxGeneration, GCCollectionMode.Forced, blocking: true, compacting: false);
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

public sealed record IsolatedBatchComparisonReport(
    int BaseRequestsAtLargestTier,
    string RuntimeVersion,
    long StopwatchFrequency,
    int OrderOffset,
    IReadOnlyList<IsolatedBatchMeasurement> Measurements,
    string InterpretationConstraint);

public sealed record IsolatedBatchMeasurement(
    string Tier,
    int Items,
    int SourceBytes,
    int ResultBytes,
    string Mode,
    int BatchSize,
    int Requests,
    int Transactions,
    double ElapsedMilliseconds,
    double TransformsPerSecond,
    double MeanMicrosecondsPerMember,
    double ManagedAllocatedBytesPerMember,
    double? TransactionP50Microseconds,
    double? TransactionP95Microseconds,
    double? TransactionP99Microseconds);
