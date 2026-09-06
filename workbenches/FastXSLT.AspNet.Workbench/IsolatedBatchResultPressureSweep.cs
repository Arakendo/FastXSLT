using System.Collections.Concurrent;
using System.Diagnostics;
using System.Text;

public static class IsolatedBatchResultPressureSweep
{
    private static readonly ResultTier[] Tiers =
    [
        new("result-items-100", 100, [1, 8, 32, 128]),
        new("result-items-1000", 1_000, [1, 8, 16]),
        new("result-items-5000", 5_000, [1, 2, 4])
    ];
    private static readonly int[] WorkerCounts = [1, 4, 8];

    public static async Task<IsolatedBatchResultPressureSweepReport> RunAsync(
        string workerPath,
        int requestedMembers,
        int orderOffset)
    {
        var members = Math.Clamp(requestedMembers, 128, 8_192);
        members -= members % 128;
        var measurements = new List<IsolatedBatchResultPressureMeasurement>();
        foreach (var tier in Tiers)
        {
            var (source, stylesheet, expected) = BuildFixture(tier.Items);
            foreach (var workerCount in WorkerCounts)
            {
                var workers = new List<FastXsltWorkerClient>(workerCount);
                try
                {
                    for (var index = 0; index < workerCount; index++)
                    {
                        workers.Add(await FastXsltWorkerClient.StartAsync(
                            workerPath,
                            $"urn:fastxslt:batch-result-pressure:{tier.Name}:source",
                            source,
                            $"urn:fastxslt:batch-result-pressure:{tier.Name}:stylesheet",
                            stylesheet));
                    }
                    for (var index = 0; index < tier.BatchSizes.Length; index++)
                    {
                        var batchSize = tier.BatchSizes[
                            (index + Math.Abs(orderOffset % tier.BatchSizes.Length)) %
                            tier.BatchSizes.Length];
                        var deliveryOrder = orderOffset % 2 == 0
                            ? new[] { "aggregate", "incremental" }
                            : new[] { "incremental", "aggregate" };
                        foreach (var delivery in deliveryOrder)
                        {
                            measurements.Add(await MeasureAsync(
                                workers,
                                tier.Name,
                                expected,
                                members,
                                batchSize,
                                delivery));
                        }
                    }
                }
                finally
                {
                    foreach (var worker in workers)
                    {
                        worker.Dispose();
                    }
                }
            }
        }
        return new IsolatedBatchResultPressureSweepReport(
            members,
            orderOffset,
            measurements,
            "Aggregate input-order framing makes time to first and final result identical within a transaction. Batch candidates are reduced as result size grows so every exact response remains below the private 1 MiB ceiling.");
    }

    private static async Task<IsolatedBatchResultPressureMeasurement> MeasureAsync(
        IReadOnlyList<FastXsltWorkerClient> workers,
        string tier,
        string expected,
        int members,
        int batchSize,
        string delivery)
    {
        var assignments = Enumerable.Range(0, workers.Count)
            .Select(_ => new List<string[]>())
            .ToArray();
        var maximumResponseBytes = 0;
        var transaction = 0;
        for (var first = 0; first < members; first += batchSize)
        {
            var count = Math.Min(batchSize, members - first);
            var identities = Enumerable.Range(first, count)
                .Select(member => $"{tier}-w{workers.Count}-b{batchSize}-{member}")
                .ToArray();
            assignments[transaction % workers.Count].Add(identities);
            maximumResponseBytes = Math.Max(
                maximumResponseBytes,
                ResponseWireBytes(identities, expected));
            transaction++;
        }

        if (maximumResponseBytes > 1024 * 1024)
        {
            throw new InvalidOperationException("Result-pressure fixture exceeded the batch response ceiling.");
        }
        foreach (var worker in workers)
        {
            RequireOutcomes(
                await worker.TransformBatchAsync([$"{tier}-w{workers.Count}-b{batchSize}-warm"]),
                expected);
        }
        Collect();
        var allocatedBefore = GC.GetTotalAllocatedBytes(precise: true);
        var before = workers.Select(worker => worker.ObserveProcess()).ToArray();
        var firstOutcomeLatencies = new ConcurrentBag<double>();
        var finalOutcomeLatencies = new ConcurrentBag<double>();
        var started = Stopwatch.StartNew();
        await Task.WhenAll(workers.Select((worker, index) => ExecuteAsync(
            worker,
            assignments[index],
            expected,
            delivery,
            firstOutcomeLatencies,
            finalOutcomeLatencies)));
        started.Stop();
        var after = workers.Select(worker => worker.ObserveProcess()).ToArray();
        var sortedFirstLatencies = firstOutcomeLatencies.Order().ToArray();
        var sortedFinalLatencies = finalOutcomeLatencies.Order().ToArray();
        return new IsolatedBatchResultPressureMeasurement(
            tier,
            Encoding.UTF8.GetByteCount(expected),
            workers.Count,
            batchSize,
            delivery,
            members,
            transaction,
            started.Elapsed.TotalMilliseconds,
            members / started.Elapsed.TotalSeconds,
            Percentile(sortedFirstLatencies, 0.50),
            Percentile(sortedFirstLatencies, 0.95),
            Percentile(sortedFirstLatencies, 0.99),
            Percentile(sortedFinalLatencies, 0.50),
            Percentile(sortedFinalLatencies, 0.95),
            Percentile(sortedFinalLatencies, 0.99),
            before.Sum(value => value.WorkingSetBytes),
            after.Sum(value => value.WorkingSetBytes),
            (GC.GetTotalAllocatedBytes(precise: true) - allocatedBefore) / (double)members,
            maximumResponseBytes,
            batchSize,
            checked(batchSize * workers.Count));
    }

    private static async Task ExecuteAsync(
        FastXsltWorkerClient worker,
        IReadOnlyList<string[]> batches,
        string expected,
        string delivery,
        ConcurrentBag<double> firstOutcomeLatencies,
        ConcurrentBag<double> finalOutcomeLatencies)
    {
        foreach (var batch in batches)
        {
            var started = Stopwatch.StartNew();
            if (delivery == "incremental")
            {
                var incremental = await worker.TransformIncrementalBatchAsync(batch);
                RequireOutcomes(incremental.Outcomes, expected);
                firstOutcomeLatencies.Add(incremental.FirstOutcomeElapsed.TotalMilliseconds);
                finalOutcomeLatencies.Add(incremental.FinalOutcomeElapsed.TotalMilliseconds);
                continue;
            }
            var outcomes = await worker.TransformBatchAsync(batch);
            started.Stop();
            RequireOutcomes(outcomes, expected);
            firstOutcomeLatencies.Add(started.Elapsed.TotalMilliseconds);
            finalOutcomeLatencies.Add(started.Elapsed.TotalMilliseconds);
        }
    }

    private static int ResponseWireBytes(IReadOnlyList<string> identities, string result)
    {
        var resultBytes = Encoding.UTF8.GetByteCount(result);
        return checked(1 + sizeof(int) + identities.Sum(identity =>
            1 + sizeof(int) + Encoding.UTF8.GetByteCount(identity) + sizeof(int) + resultBytes));
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
            if (!StringComparer.Ordinal.Equals(outcome.Result, expected))
            {
                throw new InvalidOperationException("Result-pressure sweep changed the semantic result.");
            }
        }
    }

    private static (byte[] Source, byte[] Stylesheet, string Expected) BuildFixture(int items)
    {
        var stylesheet = Encoding.UTF8.GetBytes(
            $"<xsl:stylesheet version=\"3.0\" xmlns:xsl=\"http://www.w3.org/1999/XSL/Transform\">" +
            "<xsl:output method=\"xml\" omit-xml-declaration=\"yes\"/>" +
            $"<xsl:template match=\"/\"><out><xsl:for-each select=\"1 to {items}\">" +
            "<item code=\"fixed\">payload</item></xsl:for-each></out></xsl:template>" +
            "</xsl:stylesheet>");
        var expected = new StringBuilder("<out>");
        for (var index = 0; index < items; index++)
        {
            expected.Append("<item code=\"fixed\">payload</item>");
        }
        expected.Append("</out>");
        return (Encoding.UTF8.GetBytes("<root/>"), stylesheet, expected.ToString());
    }

    private static double Percentile(double[] sorted, double percentile)
    {
        var index = (int)Math.Ceiling(percentile * sorted.Length) - 1;
        return sorted[Math.Clamp(index, 0, sorted.Length - 1)];
    }

    private static void Collect()
    {
        GC.Collect(GC.MaxGeneration, GCCollectionMode.Forced, blocking: true, compacting: false);
        GC.WaitForPendingFinalizers();
        GC.Collect(GC.MaxGeneration, GCCollectionMode.Forced, blocking: true, compacting: false);
    }

    private sealed record ResultTier(string Name, int Items, int[] BatchSizes);
}

public sealed record IsolatedBatchResultPressureSweepReport(
    int MembersPerMeasurement,
    int OrderOffset,
    IReadOnlyList<IsolatedBatchResultPressureMeasurement> Measurements,
    string InterpretationConstraint);

public sealed record IsolatedBatchResultPressureMeasurement(
    string Tier,
    int ResultBytes,
    int Workers,
    int BatchSize,
    string Delivery,
    int Members,
    int Transactions,
    double ElapsedMilliseconds,
    double TransformsPerSecond,
    double FirstOutcomeP50Milliseconds,
    double FirstOutcomeP95Milliseconds,
    double FirstOutcomeP99Milliseconds,
    double FinalOutcomeP50Milliseconds,
    double FinalOutcomeP95Milliseconds,
    double FinalOutcomeP99Milliseconds,
    long AggregateWorkingSetBeforeBytes,
    long AggregateWorkingSetAfterBytes,
    double ManagedAllocatedBytesPerMember,
    int MaximumResponseFrameBytes,
    int MaximumAmbiguousMembersPerWorkerLoss,
    int MaximumAggregateAmbiguousMembers);
