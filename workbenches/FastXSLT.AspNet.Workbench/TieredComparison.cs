using System.Diagnostics;
using System.Globalization;
using System.Text;

public static class TieredComparison
{
    private static readonly Tier[] Tiers =
    [
        new("items-5", 5, 20),
        new("items-50", 50, 4),
        new("items-500", 500, 1)
    ];

    private static readonly Tier[] TextHeavyTiers =
    [
        new("text-bytes-4096", 4_096, 20),
        new("text-bytes-65536", 65_536, 4),
        new("text-bytes-524288", 524_288, 1)
    ];

    private static readonly Tier[] ResultHeavyTiers =
    [
        new("result-items-100", 100, 10),
        new("result-items-1000", 1_000, 2),
        new("result-items-5000", 5_000, 1)
    ];

    public static async Task<TieredComparisonReport> RunAsync(
        string workerPath,
        byte[] modernStylesheet,
        byte[] dotNetStylesheet,
        byte[] dotNetLinearStylesheet,
        int requests,
        int maximumInFlight,
        int? orderSeed = null)
    {
        requests = Math.Clamp(requests, 1, 10_000);
        maximumInFlight = Math.Clamp(maximumInFlight, 1, 8);
        var measurements = new List<TierMeasurement>();
        var initializations = new List<TierInitialization>();
        var warmups = new List<TierWarmup>();
        var effectiveOrderSeed = orderSeed ?? Environment.TickCount;

        foreach (var tier in Tiers)
        {
            var source = BuildSource(tier.Items);
            var minimumTierRequests = Math.Min(10_000, checked(requests * tier.RequestMultiplier));
            var expected = $"<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>{tier.Items}.00</out>";

            var fastStart = Stopwatch.StartNew();
            using var pool = await FastXsltWorkerPool.StartAsync(
                workerPath,
                $"urn:fastxslt:benchmark:{tier.Name}:source",
                source,
                $"urn:fastxslt:benchmark:{tier.Name}:stylesheet",
                modernStylesheet,
                maximumInFlight);
            fastStart.Stop();
            initializations.Add(Initialization(
                "FastXSLT isolated",
                tier,
                fastStart.Elapsed,
                pool.ObserveProcesses().WorkingSetBytes,
                $"aggregate working set of {maximumInFlight} initialized workers"));

            var nativeStart = Stopwatch.StartNew();
            using var nativePool = NativeFastXsltPool.Create(
                $"urn:fastxslt:native-benchmark:{tier.Name}:source",
                source,
                $"urn:fastxslt:native-benchmark:{tier.Name}:stylesheet",
                modernStylesheet,
                maximumInFlight);
            nativeStart.Stop();
            initializations.Add(Initialization(
                "FastXSLT native in-process",
                tier,
                nativeStart.Elapsed,
                ObserveHostWorkingSet(),
                $"whole ASP.NET host working set after {maximumInFlight} native engines"));

            var dotNetStart = Stopwatch.StartNew();
            var dotNet = DotNetXslt1Baseline.Create(source, dotNetStylesheet);
            dotNetStart.Stop();
            initializations.Add(Initialization(
                "Microsoft XslCompiledTransform",
                tier,
                dotNetStart.Elapsed,
                ObserveHostWorkingSet(),
                "whole ASP.NET host working set after initialization"));

            var dotNetLinearStart = Stopwatch.StartNew();
            var dotNetLinear = DotNetXslt1Baseline.Create(source, dotNetLinearStylesheet);
            dotNetLinearStart.Stop();
            initializations.Add(Initialization(
                "Microsoft XslCompiledTransform (linear sibling walk)",
                tier,
                dotNetLinearStart.Elapsed,
                ObserveHostWorkingSet(),
                "whole ASP.NET host working set after cumulative initialization"));

#if SAXONCS_LOCAL
            var saxonStart = Stopwatch.StartNew();
            var saxon = SaxonCsBaseline.Create(source, modernStylesheet);
            saxonStart.Stop();
            initializations.Add(Initialization(
                "SaxonCS-HE 13.0.0",
                tier,
                saxonStart.Elapsed,
                ObserveHostWorkingSet(),
                "whole ASP.NET host working set after initialization"));
#endif

            var isolatedWarmup = await StabilizeAsync(
                "FastXSLT isolated",
                tier,
                () => pool.TransformAsync($"{tier.Name}-warm-fastxslt"),
                expected);
            warmups.Add(isolatedWarmup);
            var nativeWarmup = await StabilizeAsync(
                "FastXSLT native in-process",
                tier,
                () => nativePool.TransformAsync($"{tier.Name}-warm-native"),
                expected);
            warmups.Add(nativeWarmup);
            var dotNetOracleWarmup = StabilizeSync(
                "Microsoft XslCompiledTransform (shrinking-tail oracle)",
                tier,
                dotNet.Transform,
                expected);
            warmups.Add(dotNetOracleWarmup);
            var dotNetLinearWarmup = StabilizeSync(
                "Microsoft XslCompiledTransform (linear sibling walk)",
                tier,
                dotNetLinear.Transform,
                expected);
            warmups.Add(dotNetLinearWarmup);
#if SAXONCS_LOCAL
            var saxonStreamWarmup = StabilizeSync(
                "SaxonCS-HE 13.0.0 (byte-stream oracle)",
                tier,
                saxon.TransformStream,
                expected);
            warmups.Add(saxonStreamWarmup);
            var saxonTextWriterWarmup = StabilizeSync(
                "SaxonCS-HE 13.0.0 (selected UTF-8 TextWriter)",
                tier,
                saxon.TransformTextWriter,
                expected);
            warmups.Add(saxonTextWriterWarmup);
#endif

            var concurrencies = maximumInFlight == 1
                ? new[] { 1 }
                : new[] { 1, maximumInFlight };
            var lanes = new List<Func<ValueTask<TierMeasurement>>>();
            foreach (var currentConcurrency in concurrencies)
            {
                var concurrency = currentConcurrency;
                lanes.Add(() => new ValueTask<TierMeasurement>(MeasureAsync(
                    "FastXSLT isolated",
                    tier,
                    source.Length,
                    expected,
                    TimeBalancedRequestCount(minimumTierRequests, isolatedWarmup, concurrency),
                    concurrency,
                    identity => pool.TransformAsync(identity),
                    pool.ObserveProcesses)));
                lanes.Add(() => new ValueTask<TierMeasurement>(MeasureAsync(
                    "FastXSLT native in-process",
                    tier,
                    source.Length,
                    expected,
                    TimeBalancedRequestCount(minimumTierRequests, nativeWarmup, concurrency),
                    concurrency,
                    identity => nativePool.TransformAsync(identity),
                    observeWorkers: null)));
#if SAXONCS_LOCAL
                lanes.Add(() => new ValueTask<TierMeasurement>(MeasureSync(
                    "SaxonCS-HE 13.0.0 (byte-stream oracle)",
                    tier,
                    source.Length,
                    expected,
                    TimeBalancedRequestCount(minimumTierRequests, saxonStreamWarmup, concurrency),
                    concurrency,
                    saxon.TransformStream,
                    observeWorkers: null)));
                lanes.Add(() => new ValueTask<TierMeasurement>(MeasureSync(
                    "SaxonCS-HE 13.0.0 (selected UTF-8 TextWriter)",
                    tier,
                    source.Length,
                    expected,
                    TimeBalancedRequestCount(minimumTierRequests, saxonTextWriterWarmup, concurrency),
                    concurrency,
                    saxon.TransformTextWriter,
                    observeWorkers: null)));
#endif
                lanes.Add(() => new ValueTask<TierMeasurement>(MeasureSync(
                    "Microsoft XslCompiledTransform (shrinking-tail oracle)",
                    tier,
                    source.Length,
                    expected,
                    TimeBalancedRequestCount(minimumTierRequests, dotNetOracleWarmup, concurrency),
                    concurrency,
                    dotNet.Transform,
                    observeWorkers: null)));
                lanes.Add(() => new ValueTask<TierMeasurement>(MeasureSync(
                    "Microsoft XslCompiledTransform (linear sibling walk)",
                    tier,
                    source.Length,
                    expected,
                    TimeBalancedRequestCount(minimumTierRequests, dotNetLinearWarmup, concurrency),
                    concurrency,
                    dotNetLinear.Transform,
                    observeWorkers: null)));
            }
            Shuffle(lanes, new Random(HashCode.Combine(effectiveOrderSeed, tier.Name)));
            for (var position = 0; position < lanes.Count; position++)
            {
                var measurement = await lanes[position]();
                measurements.Add(measurement with { MeasurementPosition = position + 1 });
            }
        }

        return new TieredComparisonReport(
            requests,
            maximumInFlight,
            Environment.ProcessorCount,
            effectiveOrderSeed,
            initializations,
            warmups,
            measurements);
    }

    public static async Task<TieredComparisonReport> RunTextHeavyAsync(
        string workerPath,
        int requests,
        int maximumInFlight) => await RunFocusedAsync(
            workerPath,
            "text-heavy",
            TextHeavyTiers,
            BuildTextHeavyFixture,
            requests,
            maximumInFlight);

    public static async Task<TieredComparisonReport> RunResultHeavyAsync(
        string workerPath,
        int requests,
        int maximumInFlight) => await RunFocusedAsync(
            workerPath,
            "result-heavy",
            ResultHeavyTiers,
            BuildResultHeavyFixture,
            requests,
            maximumInFlight);

    private static async Task<TieredComparisonReport> RunFocusedAsync(
        string workerPath,
        string workload,
        IReadOnlyList<Tier> tiers,
        Func<Tier, (byte[] Source, byte[] Stylesheet, string Expected)> buildFixture,
        int requests,
        int maximumInFlight)
    {
        requests = Math.Clamp(requests, 1, 10_000);
        maximumInFlight = Math.Clamp(maximumInFlight, 1, 8);
        var measurements = new List<TierMeasurement>();
        var initializations = new List<TierInitialization>();
        var warmups = new List<TierWarmup>();

        foreach (var tier in tiers)
        {
            var (source, stylesheet, expected) = buildFixture(tier);
            var tierRequests = Math.Min(10_000, checked(requests * tier.RequestMultiplier));

            var isolatedStart = Stopwatch.StartNew();
            using var isolatedPool = await FastXsltWorkerPool.StartAsync(
                workerPath,
                $"urn:fastxslt:{workload}:{tier.Name}:source",
                source,
                $"urn:fastxslt:{workload}:{tier.Name}:stylesheet",
                stylesheet,
                maximumInFlight);
            isolatedStart.Stop();
            initializations.Add(Initialization(
                "FastXSLT isolated",
                tier,
                isolatedStart.Elapsed,
                isolatedPool.ObserveProcesses().WorkingSetBytes,
                $"aggregate working set of {maximumInFlight} initialized workers"));

            var nativeStart = Stopwatch.StartNew();
            using var nativePool = NativeFastXsltPool.Create(
                $"urn:fastxslt:native-{workload}:{tier.Name}:source",
                source,
                $"urn:fastxslt:native-{workload}:{tier.Name}:stylesheet",
                stylesheet,
                maximumInFlight);
            nativeStart.Stop();
            initializations.Add(Initialization(
                "FastXSLT native in-process",
                tier,
                nativeStart.Elapsed,
                ObserveHostWorkingSet(),
                $"whole ASP.NET host working set after {maximumInFlight} native engines"));

#if SAXONCS_LOCAL
            var saxonStart = Stopwatch.StartNew();
            var saxon = SaxonCsBaseline.Create(source, stylesheet);
            saxonStart.Stop();
            initializations.Add(Initialization(
                "SaxonCS-HE 13.0.0",
                tier,
                saxonStart.Elapsed,
                ObserveHostWorkingSet(),
                "whole ASP.NET host working set after cumulative initialization"));
#endif

            warmups.Add(await StabilizeAsync(
                "FastXSLT isolated",
                tier,
                () => isolatedPool.TransformAsync($"{tier.Name}-warm-isolated"),
                expected));
            warmups.Add(await StabilizeAsync(
                "FastXSLT native in-process",
                tier,
                () => nativePool.TransformAsync($"{tier.Name}-warm-native"),
                expected));
#if SAXONCS_LOCAL
            warmups.Add(StabilizeSync(
                "SaxonCS-HE 13.0.0 (byte stream)", tier, saxon.TransformStream, expected));
            warmups.Add(StabilizeSync(
                "SaxonCS-HE 13.0.0 (TextWriter)", tier, saxon.TransformTextWriter, expected));
#endif

            var concurrencies = maximumInFlight == 1
                ? new[] { 1 }
                : new[] { 1, maximumInFlight };
            foreach (var concurrency in concurrencies)
            {
                measurements.Add(await MeasureAsync(
                    "FastXSLT isolated",
                    tier,
                    source.Length,
                    expected,
                    tierRequests,
                    concurrency,
                    identity => isolatedPool.TransformAsync(identity),
                    isolatedPool.ObserveProcesses));
            }
            foreach (var concurrency in concurrencies)
            {
                measurements.Add(await MeasureAsync(
                    "FastXSLT native in-process",
                    tier,
                    source.Length,
                    expected,
                    tierRequests,
                    concurrency,
                    identity => nativePool.TransformAsync(identity),
                    observeWorkers: null));
            }
#if SAXONCS_LOCAL
            foreach (var concurrency in concurrencies)
            {
                measurements.Add(MeasureSync(
                    "SaxonCS-HE 13.0.0 (byte stream)",
                    tier,
                    source.Length,
                    expected,
                    tierRequests,
                    concurrency,
                    saxon.TransformStream,
                    observeWorkers: null));
                measurements.Add(MeasureSync(
                    "SaxonCS-HE 13.0.0 (TextWriter)",
                    tier,
                    source.Length,
                    expected,
                    tierRequests,
                    concurrency,
                    saxon.TransformTextWriter,
                    observeWorkers: null));
            }
#endif
        }

        return new TieredComparisonReport(
            requests,
            maximumInFlight,
            Environment.ProcessorCount,
            0,
            initializations,
            warmups,
            measurements);
    }

    private static async Task<TierWarmup> StabilizeAsync(
        string engine,
        Tier tier,
        Func<Task<string>> transform,
        string expected)
    {
        const double minimumWindowMilliseconds = 100;
        var rates = new List<double>();
        var totalCalls = 0;
        for (var window = 0; window < 30; window++)
        {
            var elapsed = Stopwatch.StartNew();
            var calls = 0;
            while ((calls < 8 || elapsed.Elapsed.TotalMilliseconds < minimumWindowMilliseconds) &&
                calls < 32_768)
            {
                var actual = await transform();
                if (!ResultsEquivalent(actual, expected))
                {
                    throw new InvalidOperationException($"{engine} failed warm-up validation.");
                }
                calls++;
            }
            elapsed.Stop();
            totalCalls += calls;
            rates.Add(calls / elapsed.Elapsed.TotalSeconds);
            if (IsStable(rates))
            {
                break;
            }
        }
        return Warmup(engine, tier, minimumWindowMilliseconds, totalCalls, rates);
    }

    private static TierWarmup StabilizeSync(
        string engine,
        Tier tier,
        Func<string> transform,
        string expected)
    {
        const double minimumWindowMilliseconds = 100;
        var rates = new List<double>();
        var totalCalls = 0;
        for (var window = 0; window < 30; window++)
        {
            var elapsed = Stopwatch.StartNew();
            var calls = 0;
            while ((calls < 8 || elapsed.Elapsed.TotalMilliseconds < minimumWindowMilliseconds) &&
                calls < 32_768)
            {
                var actual = transform();
                if (!ResultsEquivalent(actual, expected))
                {
                    throw new InvalidOperationException($"{engine} failed warm-up validation.");
                }
                calls++;
            }
            elapsed.Stop();
            totalCalls += calls;
            rates.Add(calls / elapsed.Elapsed.TotalSeconds);
            if (IsStable(rates))
            {
                break;
            }
        }
        return Warmup(engine, tier, minimumWindowMilliseconds, totalCalls, rates);
    }

    private static bool IsStable(IReadOnlyList<double> rates)
    {
        if (rates.Count < 10)
        {
            return false;
        }
        var previous = rates.Skip(rates.Count - 10).Take(5).ToArray();
        var recent = rates.Skip(rates.Count - 5).ToArray();
        var previousMedian = Median(previous);
        var recentMedian = Median(recent);
        var drift = RelativeDifference(previousMedian, recentMedian);
        return drift <= 0.05;
    }

    private static TierWarmup Warmup(
        string engine,
        Tier tier,
        double minimumWindowMilliseconds,
        int totalCalls,
        IReadOnlyList<double> rates)
    {
        var recent = rates.Skip(Math.Max(0, rates.Count - 5)).ToArray();
        var recentMedian = Median(recent);
        var previous = rates.Count >= 10
            ? rates.Skip(rates.Count - 10).Take(5).ToArray()
            : recent;
        var drift = RelativeDifference(Median(previous), recentMedian);
        var deviation = Median(recent.Select(value => Math.Abs(value - recentMedian)).ToArray());
        var relativeDeviation = recentMedian == 0 ? double.PositiveInfinity : deviation / recentMedian;
        return new TierWarmup(
            engine,
            tier.Name,
            minimumWindowMilliseconds,
            rates.Count,
            totalCalls,
            drift,
            relativeDeviation,
            rates.ToArray(),
            IsStable(rates));
    }

    private static double Median(IReadOnlyList<double> values)
    {
        var ordered = values.Order().ToArray();
        var middle = ordered.Length / 2;
        return ordered.Length % 2 == 0
            ? (ordered[middle - 1] + ordered[middle]) / 2
            : ordered[middle];
    }

    private static double RelativeDifference(double left, double right)
    {
        var scale = Math.Max(Math.Abs(left), Math.Abs(right));
        return scale == 0 ? 0 : Math.Abs(left - right) / scale;
    }

    private static void Shuffle<T>(IList<T> values, Random random)
    {
        for (var index = values.Count - 1; index > 0; index--)
        {
            var swap = random.Next(index + 1);
            (values[index], values[swap]) = (values[swap], values[index]);
        }
    }

    private static int TimeBalancedRequestCount(
        int minimumRequests,
        TierWarmup warmup,
        int concurrency)
    {
        const double targetMeasurementSeconds = 0.5;
        const int maximumRequests = 500_000;
        var recentRates = warmup.WindowThroughputsPerSecond
            .Skip(Math.Max(0, warmup.WindowThroughputsPerSecond.Count - 5))
            .ToArray();
        var sequentialRate = Median(recentRates);
        var estimated = (long)Math.Ceiling(
            sequentialRate * targetMeasurementSeconds * concurrency);
        return (int)Math.Clamp(
            Math.Max(minimumRequests, estimated),
            1,
            maximumRequests);
    }

    private static async Task<TierMeasurement> MeasureAsync(
        string engine,
        Tier tier,
        int sourceBytes,
        string expected,
        int requests,
        int concurrency,
        Func<string, Task<string>> transform,
        Func<(TimeSpan ProcessorTime, long WorkingSetBytes)>? observeWorkers)
    {
        GC.Collect(GC.MaxGeneration, GCCollectionMode.Forced, blocking: true, compacting: false);
        GC.WaitForPendingFinalizers();
        GC.Collect(GC.MaxGeneration, GCCollectionMode.Forced, blocking: true, compacting: false);
        var host = Process.GetCurrentProcess();
        host.Refresh();
        var hostCpuBefore = host.TotalProcessorTime;
        var hostWorkingSetBefore = host.WorkingSet64;
        var workersBefore = observeWorkers?.Invoke();
        var allocatedBefore = GC.GetTotalAllocatedBytes(precise: true);
        var threadPoolThreadsBefore = ThreadPool.ThreadCount;
        var latencies = new double[requests];
        var activeCalls = 0;
        var activeCallsHighWater = 0;
        var total = Stopwatch.StartNew();

        if (concurrency == 1)
        {
            for (var index = 0; index < requests; index++)
            {
                latencies[index] = await InvokeMeasured(index);
            }
        }
        else
        {
            await Parallel.ForEachAsync(
                Enumerable.Range(0, requests),
                new ParallelOptions { MaxDegreeOfParallelism = concurrency },
                async (index, _) => latencies[index] = await InvokeMeasured(index));
        }
        total.Stop();

        var allocatedBytes = GC.GetTotalAllocatedBytes(precise: true) - allocatedBefore;
        var threadPoolThreadsAfter = ThreadPool.ThreadCount;
        host.Refresh();
        var hostCpu = host.TotalProcessorTime - hostCpuBefore;
        var hostWorkingSetAfter = host.WorkingSet64;
        var workersAfter = observeWorkers?.Invoke();
        var workerCpu = workersAfter?.ProcessorTime - workersBefore?.ProcessorTime;
        var cpu = hostCpu + (workerCpu ?? TimeSpan.Zero);
        Array.Sort(latencies);

        return new TierMeasurement(
            engine,
            tier.Name,
            tier.Items,
            sourceBytes,
            Encoding.UTF8.GetByteCount(expected),
            requests,
            concurrency,
            total.Elapsed.TotalMilliseconds,
            requests / total.Elapsed.TotalSeconds,
            Percentile(latencies, 0.50),
            Percentile(latencies, 0.95),
            Percentile(latencies, 0.99),
            cpu.TotalMilliseconds,
            cpu.TotalMilliseconds / (total.Elapsed.TotalMilliseconds * Environment.ProcessorCount) * 100,
            allocatedBytes,
            hostWorkingSetBefore,
            hostWorkingSetAfter,
            workersBefore?.WorkingSetBytes,
            workersAfter?.WorkingSetBytes,
            activeCallsHighWater,
            threadPoolThreadsBefore,
            threadPoolThreadsAfter,
            0,
            "asynchronous API call and host scheduling",
            false,
            observeWorkers is null
                ? "managed allocation and whole ASP.NET host working set"
                : "managed host allocation plus aggregate isolated-worker CPU/working set");

        async Task<double> InvokeMeasured(int index)
        {
            var started = Stopwatch.GetTimestamp();
            var active = Interlocked.Increment(ref activeCalls);
            UpdateHighWater(ref activeCallsHighWater, active);
            string result;
            try
            {
                result = await transform($"{tier.Name}-{concurrency}-{index}");
            }
            finally
            {
                Interlocked.Decrement(ref activeCalls);
            }
            if (!ResultsEquivalent(result, expected))
            {
                throw new InvalidOperationException($"{engine} returned a non-equivalent result.");
            }
            return Stopwatch.GetElapsedTime(started).TotalMicroseconds;
        }
    }

    private static TierMeasurement MeasureSync(
        string engine,
        Tier tier,
        int sourceBytes,
        string expected,
        int requests,
        int concurrency,
        Func<string> transform,
        Func<(TimeSpan ProcessorTime, long WorkingSetBytes)>? observeWorkers)
    {
        GC.Collect(GC.MaxGeneration, GCCollectionMode.Forced, blocking: true, compacting: false);
        GC.WaitForPendingFinalizers();
        GC.Collect(GC.MaxGeneration, GCCollectionMode.Forced, blocking: true, compacting: false);
        var host = Process.GetCurrentProcess();
        host.Refresh();
        var hostCpuBefore = host.TotalProcessorTime;
        var hostWorkingSetBefore = host.WorkingSet64;
        var workersBefore = observeWorkers?.Invoke();
        var allocatedBefore = GC.GetTotalAllocatedBytes(precise: true);
        var threadPoolThreadsBefore = ThreadPool.ThreadCount;
        var latencies = new double[requests];
        var activeCalls = 0;
        var activeCallsHighWater = 0;
        var total = Stopwatch.StartNew();

        if (concurrency == 1)
        {
            for (var index = 0; index < requests; index++)
            {
                latencies[index] = InvokeMeasured();
            }
        }
        else
        {
            Parallel.For(
                0,
                requests,
                new ParallelOptions { MaxDegreeOfParallelism = concurrency },
                index => latencies[index] = InvokeMeasured());
        }
        total.Stop();

        var allocatedBytes = GC.GetTotalAllocatedBytes(precise: true) - allocatedBefore;
        var threadPoolThreadsAfter = ThreadPool.ThreadCount;
        host.Refresh();
        var hostCpu = host.TotalProcessorTime - hostCpuBefore;
        var hostWorkingSetAfter = host.WorkingSet64;
        var workersAfter = observeWorkers?.Invoke();
        var workerCpu = workersAfter?.ProcessorTime - workersBefore?.ProcessorTime;
        var cpu = hostCpu + (workerCpu ?? TimeSpan.Zero);
        Array.Sort(latencies);

        return new TierMeasurement(
            engine,
            tier.Name,
            tier.Items,
            sourceBytes,
            Encoding.UTF8.GetByteCount(expected),
            requests,
            concurrency,
            total.Elapsed.TotalMilliseconds,
            requests / total.Elapsed.TotalSeconds,
            Percentile(latencies, 0.50),
            Percentile(latencies, 0.95),
            Percentile(latencies, 0.99),
            cpu.TotalMilliseconds,
            cpu.TotalMilliseconds / (total.Elapsed.TotalMilliseconds * Environment.ProcessorCount) * 100,
            allocatedBytes,
            hostWorkingSetBefore,
            hostWorkingSetAfter,
            workersBefore?.WorkingSetBytes,
            workersAfter?.WorkingSetBytes,
            activeCallsHighWater,
            threadPoolThreadsBefore,
            threadPoolThreadsAfter,
            0,
            "synchronous API call with bounded Parallel.For scheduling",
            false,
            observeWorkers is null
                ? "managed allocation and whole ASP.NET host working set"
                : "managed host allocation plus aggregate isolated-worker CPU/working set");

        double InvokeMeasured()
        {
            var started = Stopwatch.GetTimestamp();
            var active = Interlocked.Increment(ref activeCalls);
            UpdateHighWater(ref activeCallsHighWater, active);
            string result;
            try
            {
                result = transform();
            }
            finally
            {
                Interlocked.Decrement(ref activeCalls);
            }
            if (!ResultsEquivalent(result, expected))
            {
                throw new InvalidOperationException($"{engine} returned a non-equivalent result.");
            }
            return Stopwatch.GetElapsedTime(started).TotalMicroseconds;
        }
    }

    private static TierInitialization Initialization(
        string engine,
        Tier tier,
        TimeSpan elapsed,
        long workingSetBytes,
        string memoryScope) => new(
            engine,
            tier.Name,
            tier.Items,
            elapsed.TotalMilliseconds,
            workingSetBytes,
            memoryScope);

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

    private static (byte[] Source, byte[] Stylesheet, string Expected) BuildTextHeavyFixture(
        Tier tier)
    {
        var text = new string('a', tier.Items);
        var stylesheet = new StringBuilder(
            "<xsl:stylesheet version=\"3.0\" xmlns:xsl=\"http://www.w3.org/1999/XSL/Transform\">" +
            "<xsl:output method=\"xml\" omit-xml-declaration=\"yes\"/>" +
            "<xsl:template match=\"/\"><out>");
        stylesheet.Append(text);
        stylesheet.Append("</out></xsl:template></xsl:stylesheet>");
        return (
            Encoding.UTF8.GetBytes("<root/>"),
            Encoding.UTF8.GetBytes(stylesheet.ToString()),
            $"<out>{text}</out>");
    }

    private static (byte[] Source, byte[] Stylesheet, string Expected) BuildResultHeavyFixture(
        Tier tier)
    {
        var stylesheet = Encoding.UTF8.GetBytes(
            $"<xsl:stylesheet version=\"3.0\" xmlns:xsl=\"http://www.w3.org/1999/XSL/Transform\">" +
            "<xsl:output method=\"xml\" omit-xml-declaration=\"yes\"/>" +
            $"<xsl:template match=\"/\"><out><xsl:for-each select=\"1 to {tier.Items}\">" +
            "<item code=\"fixed\">payload</item></xsl:for-each></out></xsl:template>" +
            "</xsl:stylesheet>");
        var expected = new StringBuilder("<out>");
        for (var index = 0; index < tier.Items; index++)
        {
            expected.Append("<item code=\"fixed\">payload</item>");
        }
        expected.Append("</out>");
        return (Encoding.UTF8.GetBytes("<root/>"), stylesheet, expected.ToString());
    }

    private static long ObserveHostWorkingSet()
    {
        using var process = Process.GetCurrentProcess();
        process.Refresh();
        return process.WorkingSet64;
    }

    private static double Percentile(double[] sorted, double percentile)
    {
        var index = (int)Math.Ceiling(percentile * sorted.Length) - 1;
        return sorted[Math.Clamp(index, 0, sorted.Length - 1)];
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
            actual.AsSpan(marker + lower.Length).SequenceEqual(expected.AsSpan(marker + upper.Length));
    }

    private static void UpdateHighWater(ref int highWater, int candidate)
    {
        var observed = Volatile.Read(ref highWater);
        while (candidate > observed)
        {
            var prior = Interlocked.CompareExchange(ref highWater, candidate, observed);
            if (prior == observed)
            {
                return;
            }
            observed = prior;
        }
    }

    private static void RequireResult(Func<string> transform, string expected, string engine)
    {
        var actual = transform();
        if (!ResultsEquivalent(actual, expected))
        {
            throw new InvalidOperationException(
                $"{engine} failed the tier warm-up result: expected {expected}, actual {actual}.");
        }
    }

    private static async Task RequireResult(
        Func<Task<string>> transform,
        string expected,
        string engine)
    {
        var actual = await transform();
        if (!ResultsEquivalent(actual, expected))
        {
            throw new InvalidOperationException(
                $"{engine} failed the tier warm-up result: expected {expected}, actual {actual}.");
        }
    }

    private sealed record Tier(string Name, int Items, int RequestMultiplier);
}

public sealed record TieredComparisonReport(
    int BaseRequestsAtLargestTier,
    int MaximumInFlight,
    int LogicalProcessors,
    int OrderSeed,
    IReadOnlyList<TierInitialization> Initializations,
    IReadOnlyList<TierWarmup> Warmups,
    IReadOnlyList<TierMeasurement> Measurements);

public sealed record TierWarmup(
    string Engine,
    string Tier,
    double MinimumWindowMilliseconds,
    int Windows,
    int TotalCalls,
    double FinalRelativeMedianDrift,
    double FinalRelativeMedianAbsoluteDeviation,
    IReadOnlyList<double> WindowThroughputsPerSecond,
    bool Stabilized);

public sealed record TierInitialization(
    string Engine,
    string Tier,
    int Items,
    double ElapsedMilliseconds,
    long WorkingSetBytes,
    string MemoryScope)
{
    public bool ComparableAcrossEngines => false;
}

public sealed record TierMeasurement(
    string Engine,
    string Tier,
    int Items,
    int SourceBytes,
    int ResultBytes,
    int Requests,
    int Concurrency,
    double ElapsedMilliseconds,
    double TransformsPerSecond,
    double P50Microseconds,
    double P95Microseconds,
    double P99Microseconds,
    double ProcessorMilliseconds,
    double NormalizedProcessorPercent,
    long ManagedAllocatedBytes,
    long HostWorkingSetBefore,
    long HostWorkingSetAfter,
    long? WorkerWorkingSetBefore,
    long? WorkerWorkingSetAfter,
    int AchievedConcurrencyHighWater,
    int ThreadPoolThreadsBefore,
    int ThreadPoolThreadsAfter,
    int MeasurementPosition,
    string MeasurementProtocol,
    bool ManagedAllocationComparableAcrossEngines,
    string ObservationScope);
