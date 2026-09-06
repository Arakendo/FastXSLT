using System.Collections.Concurrent;
using System.Diagnostics;
using System.Text;

public static class BestPracticeDeploymentComparison
{
    private static readonly (string Name, int Items, int Multiplier)[] Tiers =
    [
        ("items-5", 5, 25),
        // Diagnostic peer for items-5: retain the same member count while
        // increasing per-transform work enough to test an extremely small
        // transform against the same host concurrency envelope.
        ("items-8", 8, 25),
        ("items-50", 50, 5),
        ("items-500", 500, 1)
    ];

    private static readonly int[] BatchSizes = [1, 8, 32, 128];

    public static async Task<BestPracticeDeploymentReport> RunAsync(
        string workerPath,
        byte[] modernStylesheet,
        byte[] dotNetLinearStylesheet,
        int baseMembers,
        int maximumConcurrency,
        int? orderSeed,
        int? fixedQueuedJobs = null)
    {
        baseMembers = Math.Clamp(baseMembers, 128, 20_000);
        maximumConcurrency = Math.Clamp(maximumConcurrency, 1, 8);
        fixedQueuedJobs = fixedQueuedJobs is null
            ? null
            : Math.Clamp(fixedQueuedJobs.Value, 1, 50_000);
        var effectiveOrderSeed = orderSeed ?? Environment.TickCount;
        var measurements = new List<BestPracticeDeploymentMeasurement>();

        foreach (var tier in Tiers)
        {
            var members = fixedQueuedJobs ?? checked(baseMembers * tier.Multiplier);
            var source = BuildSource(tier.Items);
            var expected = $"<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>{tier.Items}.00</out>";
            var workers = new List<FastXsltWorkerClient>(maximumConcurrency);
            try
            {
                for (var index = 0; index < maximumConcurrency; index++)
                {
                    workers.Add(await FastXsltWorkerClient.StartAsync(
                        workerPath,
                        $"urn:fastxslt:best-practice:{tier.Name}:source",
                        source,
                        $"urn:fastxslt:best-practice:{tier.Name}:stylesheet",
                        modernStylesheet));
                }
                using var native = NativeFastXsltPool.Create(
                    $"urn:fastxslt:native-best-practice:{tier.Name}:source",
                    source,
                    $"urn:fastxslt:native-best-practice:{tier.Name}:stylesheet",
                    modernStylesheet,
                    maximumConcurrency);
                var dotNet = DotNetXslt1Baseline.Create(source, dotNetLinearStylesheet);
#if SAXONCS_LOCAL
                var saxon = SaxonCsBaseline.Create(source, modernStylesheet);
#endif

                foreach (var worker in workers)
                {
                    var warm = await worker.TransformIncrementalBatchAsync(
                        [$"{tier.Name}-isolated-warm"]);
                    RequireIncremental(warm, expected);
                }
                await Task.WhenAll(Enumerable.Range(0, maximumConcurrency).Select(index =>
                    RequireAsync(
                        native.TransformAsync($"{tier.Name}-native-warm-{index}"),
                        expected)));
                Require(dotNet.Transform(), expected);
#if SAXONCS_LOCAL
                Require(saxon.TransformTextWriter(), expected);
#endif

                var lanes = new List<Func<Task<BestPracticeDeploymentMeasurement>>>();
                foreach (var size in BatchSizes)
                {
                    var batchSize = size;
                    lanes.Add(() => MeasureIsolatedIncrementalAsync(
                        workers,
                        tier.Name,
                        tier.Items,
                        source.Length,
                        expected,
                        members,
                        batchSize));
                }
                lanes.Add(() => MeasureAsync(
                    "FastXSLT native in-process",
                    "native handle pool",
                    tier.Name,
                    tier.Items,
                    source.Length,
                    expected,
                    members,
                    maximumConcurrency,
                    index => native.TransformAsync($"{tier.Name}-native-{index}")));
                lanes.Add(() => Task.FromResult(MeasureSync(
                    "Microsoft XslCompiledTransform (linear sibling walk)",
                    "shared compiled transform",
                    tier.Name,
                    tier.Items,
                    source.Length,
                    expected,
                    members,
                    maximumConcurrency,
                    dotNet.Transform)));
#if SAXONCS_LOCAL
                lanes.Add(() => Task.FromResult(MeasureSync(
                    "SaxonCS-HE 13.0.0 (selected UTF-8 TextWriter)",
                    "shared executable; fresh transformer and TextWriter",
                    tier.Name,
                    tier.Items,
                    source.Length,
                    expected,
                    members,
                    maximumConcurrency,
                    saxon.TransformTextWriter)));
#endif
                Shuffle(lanes, new Random(HashCode.Combine(effectiveOrderSeed, tier.Name)));
                for (var position = 0; position < lanes.Count; position++)
                {
                    var measurement = await lanes[position]();
                    measurements.Add(measurement with { MeasurementPosition = position + 1 });
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

        return new BestPracticeDeploymentReport(
            baseMembers,
            maximumConcurrency,
            effectiveOrderSeed,
            Environment.ProcessorCount,
            measurements,
            fixedQueuedJobs,
            "Deployment-family comparison, not the exact-call family. When fixedQueuedJobs is present, it is the explicit independent transform-job queue length for every source-item tier. Isolated batching is incremental and non-retaining. Batch size is a private policy sweep, not a default or host setting. Microsoft remains fixture-equivalent XSLT 1.0. Allocation and working-set observations are not comparable total-memory measurements across engines.");
    }

    private static async Task<BestPracticeDeploymentMeasurement>
        MeasureIsolatedIncrementalAsync(
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
        var transaction = 0;
        for (var first = 0; first < members; first += batchSize)
        {
            var count = Math.Min(batchSize, members - first);
            var identities = Enumerable.Range(first, count)
                .Select(index => $"{tier}-isolated-b{batchSize}-{index}")
                .ToArray();
            assignments[transaction % workers.Count].Add(identities);
            transaction++;
        }

        Collect();
        var host = Process.GetCurrentProcess();
        host.Refresh();
        var hostCpuBefore = host.TotalProcessorTime;
        var hostWorkingSetBefore = host.WorkingSet64;
        var workerBefore = workers.Select(worker => worker.ObserveProcess()).ToArray();
        var allocatedBefore = GC.GetTotalAllocatedBytes(precise: true);
        var firstLatencies = new ConcurrentBag<double>();
        var finalLatencies = new ConcurrentBag<double>();
        var activeWorkers = 0;
        var activeWorkersHighWater = 0;
        long requestWireBytes = 0;
        long responseWireBytes = 0;
        var started = Stopwatch.StartNew();
        await Task.WhenAll(workers.Select((worker, index) => ExecuteWorkerAsync(
            worker,
            assignments[index],
            expected,
            firstLatencies,
            finalLatencies,
            bytes =>
            {
                Interlocked.Add(ref requestWireBytes, bytes.Request);
                Interlocked.Add(ref responseWireBytes, bytes.Response);
            },
            () =>
            {
                var active = Interlocked.Increment(ref activeWorkers);
                UpdateHighWater(ref activeWorkersHighWater, active);
            },
            () => Interlocked.Decrement(ref activeWorkers))));
        started.Stop();
        var allocated = GC.GetTotalAllocatedBytes(precise: true) - allocatedBefore;
        var workerAfter = workers.Select(worker => worker.ObserveProcess()).ToArray();
        host.Refresh();
        var cpu = host.TotalProcessorTime - hostCpuBefore +
            TimeSpan.FromMilliseconds(workerAfter.Select((value, index) =>
                (value.ProcessorTime - workerBefore[index].ProcessorTime).TotalMilliseconds).Sum());

        return Measurement(
            "FastXSLT isolated incremental",
            "bounded incremental input-order transport",
            tier,
            items,
            sourceBytes,
            expected,
            members,
            workers.Count,
            batchSize,
            transaction,
            started.Elapsed,
            firstLatencies,
            finalLatencies,
            cpu,
            allocated,
            hostWorkingSetBefore,
            host.WorkingSet64,
            workerBefore.Sum(value => value.WorkingSetBytes),
            workerAfter.Sum(value => value.WorkingSetBytes),
            requestWireBytes,
            responseWireBytes,
            batchSize,
            checked(batchSize * workers.Count),
            activeWorkersHighWater);
    }

    private static async Task ExecuteWorkerAsync(
        FastXsltWorkerClient worker,
        IReadOnlyList<string[]> batches,
        string expected,
        ConcurrentBag<double> firstLatencies,
        ConcurrentBag<double> finalLatencies,
        Action<(int Request, int Response)> observeBytes,
        Action onTransactionStart,
        Action onTransactionEnd)
    {
        foreach (var batch in batches)
        {
            onTransactionStart();
            IsolatedWorkerIncrementalBatchResult result;
            try
            {
                result = await worker.ConsumeIncrementalBatchAsync(batch, outcome =>
                {
                    RequireOutcome(outcome, expected);
                    return ValueTask.CompletedTask;
                });
            }
            finally
            {
                onTransactionEnd();
            }
            firstLatencies.Add(result.FirstOutcomeElapsed.TotalMicroseconds);
            finalLatencies.Add(result.FinalOutcomeElapsed.TotalMicroseconds);
            observeBytes((result.EncodedRequestBytes, result.EncodedResponseBytes));
        }
    }

    private static async Task<BestPracticeDeploymentMeasurement> MeasureAsync(
        string engine,
        string mode,
        string tier,
        int items,
        int sourceBytes,
        string expected,
        int members,
        int concurrency,
        Func<int, Task<string>> transform)
    {
        Collect();
        var host = Process.GetCurrentProcess();
        host.Refresh();
        var cpuBefore = host.TotalProcessorTime;
        var workingSetBefore = host.WorkingSet64;
        var allocatedBefore = GC.GetTotalAllocatedBytes(precise: true);
        var latencies = new double[members];
        var activeCalls = 0;
        var activeCallsHighWater = 0;
        var started = Stopwatch.StartNew();
        await Parallel.ForEachAsync(
            Enumerable.Range(0, members),
            new ParallelOptions { MaxDegreeOfParallelism = concurrency },
            async (index, _) =>
            {
                var call = Stopwatch.GetTimestamp();
                var active = Interlocked.Increment(ref activeCalls);
                UpdateHighWater(ref activeCallsHighWater, active);
                try
                {
                    Require(await transform(index), expected);
                    latencies[index] = Stopwatch.GetElapsedTime(call).TotalMicroseconds;
                }
                finally
                {
                    Interlocked.Decrement(ref activeCalls);
                }
            });
        started.Stop();
        var allocated = GC.GetTotalAllocatedBytes(precise: true) - allocatedBefore;
        host.Refresh();
        var latency = Sorted(latencies);
        return Measurement(
            engine, mode, tier, items, sourceBytes, expected, members, concurrency,
            1, members, started.Elapsed, latency, latency,
            host.TotalProcessorTime - cpuBefore, allocated, workingSetBefore,
            host.WorkingSet64, null, null, 0, 0, 1, concurrency,
            activeCallsHighWater);
    }

    private static BestPracticeDeploymentMeasurement MeasureSync(
        string engine,
        string mode,
        string tier,
        int items,
        int sourceBytes,
        string expected,
        int members,
        int concurrency,
        Func<string> transform)
    {
        Collect();
        var host = Process.GetCurrentProcess();
        host.Refresh();
        var cpuBefore = host.TotalProcessorTime;
        var workingSetBefore = host.WorkingSet64;
        var allocatedBefore = GC.GetTotalAllocatedBytes(precise: true);
        var latencies = new double[members];
        var activeCalls = 0;
        var activeCallsHighWater = 0;
        var started = Stopwatch.StartNew();
        Parallel.For(
            0,
            members,
            new ParallelOptions { MaxDegreeOfParallelism = concurrency },
            index =>
            {
                var call = Stopwatch.GetTimestamp();
                var active = Interlocked.Increment(ref activeCalls);
                UpdateHighWater(ref activeCallsHighWater, active);
                try
                {
                    Require(transform(), expected);
                    latencies[index] = Stopwatch.GetElapsedTime(call).TotalMicroseconds;
                }
                finally
                {
                    Interlocked.Decrement(ref activeCalls);
                }
            });
        started.Stop();
        var allocated = GC.GetTotalAllocatedBytes(precise: true) - allocatedBefore;
        host.Refresh();
        var latency = Sorted(latencies);
        return Measurement(
            engine, mode, tier, items, sourceBytes, expected, members, concurrency,
            1, members, started.Elapsed, latency, latency,
            host.TotalProcessorTime - cpuBefore, allocated, workingSetBefore,
            host.WorkingSet64, null, null, 0, 0, 1, concurrency,
            activeCallsHighWater);
    }

    private static BestPracticeDeploymentMeasurement Measurement(
        string engine,
        string mode,
        string tier,
        int items,
        int sourceBytes,
        string expected,
        int members,
        int concurrency,
        int batchSize,
        int transactions,
        TimeSpan elapsed,
        IEnumerable<double> firstLatencies,
        IEnumerable<double> finalLatencies,
        TimeSpan cpu,
        long allocated,
        long hostWorkingSetBefore,
        long hostWorkingSetAfter,
        long? workerWorkingSetBefore,
        long? workerWorkingSetAfter,
        long requestWireBytes,
        long responseWireBytes,
        int ambiguousPerWorkerLoss,
        int aggregateAmbiguous,
        int achievedConcurrencyHighWater) => new(
            engine,
            mode,
            tier,
            items,
            sourceBytes,
            Encoding.UTF8.GetByteCount(expected),
            members,
            concurrency,
            batchSize,
            transactions,
            elapsed.TotalMilliseconds,
            members / elapsed.TotalSeconds,
            Percentile(firstLatencies, 0.50),
            Percentile(firstLatencies, 0.95),
            Percentile(finalLatencies, 0.50),
            Percentile(finalLatencies, 0.95),
            cpu.TotalMilliseconds,
            allocated / (double)members,
            hostWorkingSetBefore,
            hostWorkingSetAfter,
            workerWorkingSetBefore,
            workerWorkingSetAfter,
            requestWireBytes,
            responseWireBytes,
            ambiguousPerWorkerLoss,
            aggregateAmbiguous,
            achievedConcurrencyHighWater,
            0);

    private static double Percentile(IEnumerable<double> values, double percentile)
    {
        var sorted = values is double[] array ? array : values.Order().ToArray();
        var index = (int)Math.Ceiling(percentile * sorted.Length) - 1;
        return sorted[Math.Clamp(index, 0, sorted.Length - 1)];
    }

    private static double[] Sorted(double[] values)
    {
        Array.Sort(values);
        return values;
    }

    private static void RequireIncremental(
        IsolatedWorkerIncrementalBatchResult result,
        string expected)
    {
        foreach (var outcome in result.Outcomes)
        {
            RequireOutcome(outcome, expected);
        }
    }

    private static void RequireOutcome(IsolatedWorkerBatchOutcome outcome, string expected)
    {
        if (outcome.Failure is not null)
        {
            throw outcome.Failure;
        }
        Require(outcome.Result, expected);
    }

    private static async Task RequireAsync(Task<string> result, string expected) =>
        Require(await result, expected);

    private static void Require(string? actual, string expected)
    {
        if (actual is null || !ResultsEquivalent(actual, expected))
        {
            throw new InvalidOperationException("Best-practice lane changed the semantic result.");
        }
    }

    private static bool ResultsEquivalent(string actual, string expected)
    {
        if (StringComparer.Ordinal.Equals(actual, expected))
        {
            return true;
        }
        const string upper = "encoding=\"UTF-8\"";
        const string lower = "encoding=\"utf-8\"";
        var marker = expected.IndexOf(upper, StringComparison.Ordinal);
        return marker >= 0 &&
            actual.Length == expected.Length &&
            actual.AsSpan(0, marker).SequenceEqual(expected.AsSpan(0, marker)) &&
            actual.AsSpan(marker, lower.Length).SequenceEqual(lower) &&
            actual.AsSpan(marker + lower.Length).SequenceEqual(
                expected.AsSpan(marker + upper.Length));
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

    private static void Shuffle<T>(IList<T> values, Random random)
    {
        for (var index = values.Count - 1; index > 0; index--)
        {
            var swap = random.Next(index + 1);
            (values[index], values[swap]) = (values[swap], values[index]);
        }
    }

    private static void UpdateHighWater(ref int highWater, int active)
    {
        while (true)
        {
            var observed = Volatile.Read(ref highWater);
            if (active <= observed ||
                Interlocked.CompareExchange(ref highWater, active, observed) == observed)
            {
                return;
            }
        }
    }

    private static void Collect()
    {
        GC.Collect(GC.MaxGeneration, GCCollectionMode.Forced, blocking: true, compacting: false);
        GC.WaitForPendingFinalizers();
        GC.Collect(GC.MaxGeneration, GCCollectionMode.Forced, blocking: true, compacting: false);
    }
}

public sealed record BestPracticeDeploymentReport(
    int BaseMembersAtLargestTier,
    int MaximumConcurrency,
    int OrderSeed,
    int LogicalProcessors,
    IReadOnlyList<BestPracticeDeploymentMeasurement> Measurements,
    int? FixedQueuedJobs,
    string InterpretationConstraint);

public sealed record BestPracticeDeploymentMeasurement(
    string Engine,
    string Mode,
    string Tier,
    int Items,
    int SourceBytes,
    int ResultBytes,
    int Members,
    int Concurrency,
    int BatchSize,
    int Transactions,
    double ElapsedMilliseconds,
    double TransformsPerSecond,
    double FirstResultP50Microseconds,
    double FirstResultP95Microseconds,
    double FinalResultP50Microseconds,
    double FinalResultP95Microseconds,
    double ProcessorMilliseconds,
    double ManagedAllocatedBytesPerMember,
    long HostWorkingSetBeforeBytes,
    long HostWorkingSetAfterBytes,
    long? WorkerWorkingSetBeforeBytes,
    long? WorkerWorkingSetAfterBytes,
    long RequestWireBytes,
    long ResponseWireBytes,
    int MaximumAmbiguousMembersPerWorkerLoss,
    int MaximumAggregateAmbiguousMembers,
    int AchievedConcurrencyHighWater,
    int MeasurementPosition)
{
    public int QueuedJobs => Members;
}
