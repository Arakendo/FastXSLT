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
        int requests,
        int maximumInFlight)
    {
        requests = Math.Clamp(requests, 1, 10_000);
        maximumInFlight = Math.Clamp(maximumInFlight, 1, 8);
        var measurements = new List<TierMeasurement>();
        var initializations = new List<TierInitialization>();

        foreach (var tier in Tiers)
        {
            var source = BuildSource(tier.Items);
            var tierRequests = Math.Min(10_000, checked(requests * tier.RequestMultiplier));
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

            await RequireResult(
                () => pool.TransformAsync($"{tier.Name}-warm-fastxslt"), expected, "FastXSLT");
            await RequireResult(
                () => nativePool.TransformAsync($"{tier.Name}-warm-native"),
                expected,
                "FastXSLT native");
            RequireResult(dotNet.Transform, expected, "Microsoft XslCompiledTransform");
#if SAXONCS_LOCAL
            RequireResult(saxon.Transform, expected, "SaxonCS");
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
                    identity => pool.TransformAsync(identity),
                    pool.ObserveProcesses));
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
                measurements.Add(await MeasureAsync(
                    "SaxonCS-HE 13.0.0",
                    tier,
                    source.Length,
                    expected,
                    tierRequests,
                    concurrency,
                    _ => Task.FromResult(saxon.Transform()),
                    observeWorkers: null));
            }
#endif
            foreach (var concurrency in concurrencies)
            {
                measurements.Add(await MeasureAsync(
                    "Microsoft XslCompiledTransform",
                    tier,
                    source.Length,
                    expected,
                    tierRequests,
                    concurrency,
                    _ => Task.FromResult(dotNet.Transform()),
                    observeWorkers: null));
            }
        }

        return new TieredComparisonReport(
            requests,
            maximumInFlight,
            Environment.ProcessorCount,
            initializations,
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

            await RequireResult(
                () => isolatedPool.TransformAsync($"{tier.Name}-warm-isolated"),
                expected,
                "FastXSLT isolated");
            await RequireResult(
                () => nativePool.TransformAsync($"{tier.Name}-warm-native"),
                expected,
                "FastXSLT native");

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
        }

        return new TieredComparisonReport(
            requests,
            maximumInFlight,
            Environment.ProcessorCount,
            initializations,
            measurements);
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
        var latencies = new double[requests];
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
            observeWorkers is null
                ? "managed allocation and whole ASP.NET host working set"
                : "managed host allocation plus aggregate isolated-worker CPU/working set");

        async Task<double> InvokeMeasured(int index)
        {
            var started = Stopwatch.GetTimestamp();
            var result = await transform($"{tier.Name}-{concurrency}-{index}");
            var elapsed = Stopwatch.GetElapsedTime(started).TotalMicroseconds;
            if (!StringComparer.Ordinal.Equals(CanonicalizeEncoding(result), expected))
            {
                throw new InvalidOperationException($"{engine} returned a non-equivalent result.");
            }
            return elapsed;
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

    private static string CanonicalizeEncoding(string result) =>
        result.Replace("encoding=\"utf-8\"", "encoding=\"UTF-8\"", StringComparison.Ordinal);

    private static void RequireResult(Func<string> transform, string expected, string engine)
    {
        var actual = transform();
        if (!StringComparer.Ordinal.Equals(CanonicalizeEncoding(actual), expected))
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
        if (!StringComparer.Ordinal.Equals(CanonicalizeEncoding(actual), expected))
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
    IReadOnlyList<TierInitialization> Initializations,
    IReadOnlyList<TierMeasurement> Measurements);

public sealed record TierInitialization(
    string Engine,
    string Tier,
    int Items,
    double ElapsedMilliseconds,
    long WorkingSetBytes,
    string MemoryScope);

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
    string ObservationScope);
