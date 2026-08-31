using System.Diagnostics;
using System.Text;

public static class NativeRegistryBurstExperiment
{
    private const string LargeResultPrefix =
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>";
    private const string LargeResultSuffix = "</out>";

    public static async Task<NativeRegistryBurstReport> RunAsync(
        byte[] ordinaryStylesheet,
        int concurrency,
        int delayedFailures,
        int largeOutcomes,
        int largePayloadBytes)
    {
        concurrency = Math.Clamp(concurrency, 1, 32);
        delayedFailures = Math.Clamp(delayedFailures, 1, 4_096);
        largeOutcomes = Math.Clamp(largeOutcomes, 1, 32);
        largePayloadBytes = Math.Clamp(largePayloadBytes, 1, 950_000);

        var checkpoints = new List<NativeRegistryPressureCheckpoint>();
        var settlement = new List<NativeRegistrySettlementSample>();
        var retainedFailures = new List<NativeFastXsltClient.NativeRetainedOutcome>(
            delayedFailures);
        var retainedLargeResults = new List<NativeFastXsltClient.NativeRetainedOutcome>(
            largeOutcomes);
        var experiment = Stopwatch.StartNew();
        var baseline = Observe("baseline", experiment.Elapsed.TotalMilliseconds);
        checkpoints.Add(baseline);

        using var ordinaryPool = NativeFastXsltPool.Create(
            "urn:fastxslt:registry-burst:ordinary-source",
            BuildCountSource(500),
            "urn:fastxslt:registry-burst:ordinary-stylesheet",
            ordinaryStylesheet,
            concurrency);
        NativeFastXsltPool? largePool = null;
        try
        {
            checkpoints.Add(Observe(
                "ordinary-pool-prepared",
                experiment.Elapsed.TotalMilliseconds));

            var activeStarted = Stopwatch.StartNew();
            var activeFailures = await ordinaryPool.ExerciseActiveControlBurstAsync(
                "registry-active",
                TimeSpan.FromSeconds(5),
                () => checkpoints.Add(Observe(
                    "active-controls-at-first-charge",
                    experiment.Elapsed.TotalMilliseconds)));
            activeStarted.Stop();
            if (activeFailures.Count != concurrency ||
                activeFailures.Any(failure => failure.RequestId is null))
            {
                throw new InvalidOperationException(
                    "Active-control burst did not conserve request identity.");
            }
            checkpoints.Add(Observe(
                "active-controls-released",
                experiment.Elapsed.TotalMilliseconds));

            var failureStarted = Stopwatch.StartNew();
            var failureTasks = Enumerable.Range(0, delayedFailures)
                .Select(_ => ordinaryPool.TransformRetainedAsync(string.Empty));
            retainedFailures.AddRange(await Task.WhenAll(failureTasks));
            foreach (var outcome in retainedFailures)
            {
                var failure = outcome.ReadFailure();
                if (failure.Code != "FXWB0003" ||
                    failure.Category != "invalid" ||
                    failure.RequestId is not null)
                {
                    throw new InvalidOperationException(
                        "Retained failure burst changed its structured diagnostic.");
                }
            }
            failureStarted.Stop();
            checkpoints.Add(Observe(
                "delayed-structured-failures",
                experiment.Elapsed.TotalMilliseconds));

            var largeSource = BuildLargeSource(largePayloadBytes);
            var largeStylesheet = BuildLargeResultStylesheet();
            var largeEngineCount = Math.Min(concurrency, 8);
            var largeStarted = Stopwatch.StartNew();
            largePool = NativeFastXsltPool.Create(
                "urn:fastxslt:registry-burst:large-source",
                largeSource,
                "urn:fastxslt:registry-burst:large-stylesheet",
                largeStylesheet,
                largeEngineCount);
            var largeTasks = Enumerable.Range(0, largeOutcomes)
                .Select(index => largePool.TransformRetainedAsync($"registry-large-{index}"));
            retainedLargeResults.AddRange(await Task.WhenAll(largeTasks));
            foreach (var outcome in retainedLargeResults)
            {
                RequireLargeResult(outcome.ReadResult(), largePayloadBytes);
            }
            largeStarted.Stop();
            checkpoints.Add(Observe(
                "delayed-near-limit-results",
                experiment.Elapsed.TotalMilliseconds));

            foreach (var outcome in retainedFailures)
            {
                outcome.Dispose();
            }
            retainedFailures.Clear();
            checkpoints.Add(Observe(
                "structured-failures-released",
                experiment.Elapsed.TotalMilliseconds));

            foreach (var outcome in retainedLargeResults)
            {
                outcome.Dispose();
            }
            retainedLargeResults.Clear();
            checkpoints.Add(Observe(
                "near-limit-results-released",
                experiment.Elapsed.TotalMilliseconds));

            largePool.Dispose();
            largePool = null;
            ordinaryPool.Dispose();
            checkpoints.Add(Observe(
                "all-experiment-handles-released",
                experiment.Elapsed.TotalMilliseconds));

            var priorDelay = 0;
            foreach (var delay in new[] { 0, 10, 50, 100, 250, 1_000 })
            {
                if (delay > priorDelay)
                {
                    await Task.Delay(delay - priorDelay);
                }
                settlement.Add(new NativeRegistrySettlementSample(
                    delay,
                    Observe($"settlement-{delay}-ms", experiment.Elapsed.TotalMilliseconds)));
                priorDelay = delay;
            }

            var all = checkpoints.Concat(settlement.Select(value => value.Checkpoint)).ToArray();
            var highWater = new NativeRegistryHighWater(
                all.Max(value => value.Registry.EngineHandles),
                all.Max(value => value.Registry.ControlHandles),
                all.Max(value => value.Registry.OutcomeHandles),
                all.Max(value => value.Registry.OutcomePayloadBytes));
            var final = settlement[^1].Checkpoint.Registry;
            return new NativeRegistryBurstReport(
                concurrency,
                delayedFailures,
                largeOutcomes,
                largePayloadBytes,
                largeSource.Length,
                largeStylesheet.Length,
                activeStarted.Elapsed.TotalMilliseconds,
                failureStarted.Elapsed.TotalMilliseconds,
                largeStarted.Elapsed.TotalMilliseconds,
                highWater,
                checkpoints,
                settlement,
                final == baseline.Registry,
                ObservationScope:
                    "real active controls, retained structured failures, retained near-limit results, and whole ASP.NET process memory");
        }
        finally
        {
            foreach (var outcome in retainedFailures)
            {
                outcome.Dispose();
            }
            foreach (var outcome in retainedLargeResults)
            {
                outcome.Dispose();
            }
            largePool?.Dispose();
        }
    }

    private static void RequireLargeResult(string result, int payloadBytes)
    {
        if (!result.StartsWith(LargeResultPrefix, StringComparison.Ordinal) ||
            !result.EndsWith(LargeResultSuffix, StringComparison.Ordinal) ||
            result.Length != LargeResultPrefix.Length + payloadBytes + LargeResultSuffix.Length ||
            result.AsSpan(LargeResultPrefix.Length, payloadBytes).IndexOfAnyExcept('x') >= 0)
        {
            throw new InvalidOperationException("Near-limit result semantic sentinel failed.");
        }
    }

    private static NativeRegistryPressureCheckpoint Observe(string phase, double elapsedMilliseconds)
    {
        using var process = Process.GetCurrentProcess();
        process.Refresh();
        return new NativeRegistryPressureCheckpoint(
            phase,
            elapsedMilliseconds,
            NativeFastXsltClient.ObserveRegistry(),
            process.WorkingSet64,
            process.PrivateMemorySize64,
            GC.GetTotalMemory(forceFullCollection: false));
    }

    private static byte[] BuildCountSource(int items)
    {
        var source = new StringBuilder("<?xml version=\"1.0\"?><order>");
        for (var index = 0; index < items; index++)
        {
            source.Append("<order-item price=\"1.00\" qty=\"1\"/>");
        }
        source.Append("</order>");
        return Encoding.UTF8.GetBytes(source.ToString());
    }

    private static byte[] BuildLargeSource(int payloadBytes) =>
        Encoding.UTF8.GetBytes($"<root><payload>{new string('x', payloadBytes)}</payload></root>");

    private static byte[] BuildLargeResultStylesheet() => Encoding.UTF8.GetBytes(
        "<xsl:stylesheet xmlns:xsl=\"http://www.w3.org/1999/XSL/Transform\" version=\"3.0\">" +
        "<xsl:template match=\"/\"><out><xsl:value-of select=\"/root/payload\"/>" +
        "</out></xsl:template></xsl:stylesheet>");
}

public sealed record NativeRegistryBurstReport(
    int Concurrency,
    int DelayedFailures,
    int LargeOutcomes,
    int LargePayloadBytes,
    int LargeSourceBytes,
    int LargeStylesheetBytes,
    double ActiveControlBurstMilliseconds,
    double FailureBurstMilliseconds,
    double LargeResultBurstMilliseconds,
    NativeRegistryHighWater LegitimateHighWater,
    IReadOnlyList<NativeRegistryPressureCheckpoint> Checkpoints,
    IReadOnlyList<NativeRegistrySettlementSample> Settlement,
    bool LogicalRegistryReturnedToBaseline,
    string ObservationScope);
