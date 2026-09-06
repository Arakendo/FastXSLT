using System.Diagnostics;
using System.Text;

public static class IsolatedBatchWorkerSweep
{
    private static readonly (string Name, int Items)[] Tiers =
    [
        ("items-5", 5),
        ("items-50", 50),
        ("items-500", 500)
    ];
    private static readonly int[] WorkerCounts = [1, 2, 4, 8];
    private static readonly int[] BatchSizes = [1, 8, 32, 128];

    public static async Task<IsolatedBatchWorkerSweepReport> RunAsync(
        string workerPath,
        byte[] stylesheet,
        int requestedMembers,
        int orderOffset)
    {
        var members = Math.Clamp(requestedMembers, 128, 65_536);
        members -= members % 128;
        orderOffset = Math.Abs(orderOffset % BatchSizes.Length);
        var measurements = new List<IsolatedBatchWorkerSweepMeasurement>();
        foreach (var tier in Tiers)
        {
            var source = BuildSource(tier.Items);
            var expected = $"<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>{tier.Items}.00</out>";
            foreach (var workerCount in WorkerCounts)
            {
                var workers = new List<FastXsltWorkerClient>(workerCount);
                try
                {
                    for (var index = 0; index < workerCount; index++)
                    {
                        workers.Add(await FastXsltWorkerClient.StartAsync(
                            workerPath,
                            $"urn:fastxslt:batch-sweep:{tier.Name}:source",
                            source,
                            $"urn:fastxslt:batch-sweep:{tier.Name}:stylesheet",
                            stylesheet));
                    }
                    for (var sizeIndex = 0; sizeIndex < BatchSizes.Length; sizeIndex++)
                    {
                        var batchSize = BatchSizes[(sizeIndex + orderOffset) % BatchSizes.Length];
                        measurements.Add(await MeasureAsync(
                            workers,
                            tier.Name,
                            tier.Items,
                            source.Length,
                            expected,
                            members,
                            batchSize));
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
        return new IsolatedBatchWorkerSweepReport(
            members,
            Environment.Version.ToString(),
            Environment.ProcessorCount,
            orderOffset,
            measurements,
            "Worker processes and retained generations are created before each four-size sweep. Working set is an observed process boundary value, not exact FastXSLT ownership or peak memory. A single worker loss affects at most one active batch; aggregate ambiguity assumes simultaneous loss of every worker.");
    }

    private static async Task<IsolatedBatchWorkerSweepMeasurement> MeasureAsync(
        IReadOnlyList<FastXsltWorkerClient> workers,
        string tier,
        int items,
        int sourceBytes,
        string expected,
        int members,
        int batchSize)
    {
        var assignments = Enumerable.Range(0, workers.Count)
            .Select(_ => new List<string[]>())
            .ToArray();
        long totalRequestWireBytes = 0;
        long totalResponseWireBytes = 0;
        var maximumRequestFrameBytes = 0;
        var maximumResponseFrameBytes = 0;
        var transaction = 0;
        for (var first = 0; first < members; first += batchSize)
        {
            var count = Math.Min(batchSize, members - first);
            var identities = Enumerable.Range(first, count)
                .Select(index => $"{tier}-w{workers.Count}-b{batchSize}-{index}")
                .ToArray();
            assignments[transaction % workers.Count].Add(identities);
            var requestBytes = RequestWireBytes(identities);
            var responseBytes = ResponseWireBytes(identities, expected);
            totalRequestWireBytes += requestBytes;
            totalResponseWireBytes += responseBytes;
            maximumRequestFrameBytes = Math.Max(maximumRequestFrameBytes, requestBytes);
            maximumResponseFrameBytes = Math.Max(maximumResponseFrameBytes, responseBytes);
            transaction++;
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
        var started = Stopwatch.StartNew();
        await Task.WhenAll(workers.Select((worker, index) => ExecuteAsync(
            worker,
            assignments[index],
            expected)));
        started.Stop();
        var after = workers.Select(worker => worker.ObserveProcess()).ToArray();
        var cpuMilliseconds = after.Select((value, index) =>
            (value.ProcessorTime - before[index].ProcessorTime).TotalMilliseconds).Sum();
        return new IsolatedBatchWorkerSweepMeasurement(
            tier,
            items,
            sourceBytes,
            Encoding.UTF8.GetByteCount(expected),
            workers.Count,
            batchSize,
            members,
            transaction,
            started.Elapsed.TotalMilliseconds,
            members / started.Elapsed.TotalSeconds,
            cpuMilliseconds,
            cpuMilliseconds / started.Elapsed.TotalMilliseconds,
            before.Sum(value => value.WorkingSetBytes),
            after.Sum(value => value.WorkingSetBytes),
            (GC.GetTotalAllocatedBytes(precise: true) - allocatedBefore) / (double)members,
            totalRequestWireBytes,
            totalResponseWireBytes,
            maximumRequestFrameBytes,
            maximumResponseFrameBytes,
            checked(batchSize * workers.Count),
            batchSize,
            checked(batchSize * workers.Count));
    }

    private static async Task ExecuteAsync(
        FastXsltWorkerClient worker,
        IReadOnlyList<string[]> batches,
        string expected)
    {
        foreach (var batch in batches)
        {
            RequireOutcomes(await worker.TransformBatchAsync(batch), expected);
        }
    }

    private static int RequestWireBytes(IReadOnlyList<string> identities) =>
        checked(1 + sizeof(int) + identities.Sum(identity =>
            sizeof(int) + Encoding.UTF8.GetByteCount(identity)));

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
                throw new InvalidOperationException("Batch worker sweep changed the semantic result.");
            }
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

    private static void Collect()
    {
        GC.Collect(GC.MaxGeneration, GCCollectionMode.Forced, blocking: true, compacting: false);
        GC.WaitForPendingFinalizers();
        GC.Collect(GC.MaxGeneration, GCCollectionMode.Forced, blocking: true, compacting: false);
    }
}

public sealed record IsolatedBatchWorkerSweepReport(
    int MembersPerMeasurement,
    string RuntimeVersion,
    int LogicalProcessors,
    int OrderOffset,
    IReadOnlyList<IsolatedBatchWorkerSweepMeasurement> Measurements,
    string InterpretationConstraint);

public sealed record IsolatedBatchWorkerSweepMeasurement(
    string Tier,
    int Items,
    int SourceBytes,
    int ResultBytes,
    int Workers,
    int BatchSize,
    int Members,
    int Transactions,
    double ElapsedMilliseconds,
    double TransformsPerSecond,
    double AggregateWorkerCpuMilliseconds,
    double EffectiveBusyCores,
    long AggregateWorkingSetBeforeBytes,
    long AggregateWorkingSetAfterBytes,
    double ManagedAllocatedBytesPerMember,
    long TotalRequestWireBytes,
    long TotalResponseWireBytes,
    int MaximumRequestFrameBytes,
    int MaximumResponseFrameBytes,
    int MaximumOutstandingMembers,
    int MaximumAmbiguousMembersPerWorkerLoss,
    int MaximumAggregateAmbiguousMembers);
