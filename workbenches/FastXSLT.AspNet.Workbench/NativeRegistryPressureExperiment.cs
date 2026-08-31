using System.Diagnostics;
using System.Text;

public static class NativeRegistryPressureExperiment
{
    public static async Task<NativeRegistryPressureReport> RunAsync(
        byte[] stylesheet,
        int items,
        int concurrency,
        int generations,
        int delayedOutcomes)
    {
        items = Math.Clamp(items, 1, 5_000);
        concurrency = Math.Clamp(concurrency, 1, 32);
        generations = Math.Clamp(generations, 2, 3);
        delayedOutcomes = Math.Clamp(delayedOutcomes, 1, 4_096);

        var source = BuildSource(items);
        var expected = $"<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>{items}.00</out>";
        var pools = new List<NativeFastXsltPool>(generations);
        var retainedOutcomes = new List<NativeFastXsltClient.NativeRetainedOutcome>(
            delayedOutcomes);
        var checkpoints = new List<NativeRegistryPressureCheckpoint>();
        var settlement = new List<NativeRegistrySettlementSample>();
        var experiment = Stopwatch.StartNew();
        var baseline = Observe("baseline", experiment.Elapsed.TotalMilliseconds);
        checkpoints.Add(baseline);

        try
        {
            for (var generation = 0; generation < generations; generation++)
            {
                pools.Add(NativeFastXsltPool.Create(
                    $"urn:fastxslt:registry-pressure:source:g{generation}",
                    source,
                    $"urn:fastxslt:registry-pressure:stylesheet:g{generation}",
                    stylesheet,
                    concurrency));
                checkpoints.Add(Observe(
                    $"generation-{generation + 1}-prepared",
                    experiment.Elapsed.TotalMilliseconds));
            }

            var current = pools[^1];
            var outcomeTasks = Enumerable.Range(0, delayedOutcomes)
                .Select(index => current.TransformRetainedAsync($"registry-pressure-{index}"));
            retainedOutcomes.AddRange(await Task.WhenAll(outcomeTasks));
            foreach (var outcome in retainedOutcomes)
            {
                var actual = outcome.ReadResult();
                if (!StringComparer.Ordinal.Equals(actual, expected))
                {
                    throw new InvalidOperationException(
                        $"Registry pressure semantic sentinel failed: expected {expected}, actual {actual}.");
                }
            }
            checkpoints.Add(Observe(
                "delayed-valid-outcomes",
                experiment.Elapsed.TotalMilliseconds));

            for (var generation = 0; generation < pools.Count - 1; generation++)
            {
                pools[generation].Dispose();
            }
            checkpoints.Add(Observe(
                "old-generations-retired",
                experiment.Elapsed.TotalMilliseconds));

            foreach (var outcome in retainedOutcomes)
            {
                outcome.Dispose();
            }
            retainedOutcomes.Clear();
            checkpoints.Add(Observe(
                "delayed-outcomes-released",
                experiment.Elapsed.TotalMilliseconds));

            pools[^1].Dispose();
            pools.Clear();
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
                var sample = Observe(
                    $"settlement-{delay}-ms",
                    experiment.Elapsed.TotalMilliseconds);
                settlement.Add(new NativeRegistrySettlementSample(delay, sample));
                priorDelay = delay;
            }

            var highWater = new NativeRegistryHighWater(
                checkpoints.Max(value => value.Registry.EngineHandles),
                checkpoints.Max(value => value.Registry.ControlHandles),
                checkpoints.Max(value => value.Registry.OutcomeHandles),
                checkpoints.Max(value => value.Registry.OutcomePayloadBytes));
            var final = settlement[^1].Checkpoint;
            return new NativeRegistryPressureReport(
                items,
                concurrency,
                generations,
                delayedOutcomes,
                source.Length,
                stylesheet.Length,
                checked((long)concurrency * generations * (source.Length + stylesheet.Length)),
                expected,
                highWater,
                checkpoints,
                settlement,
                final.Registry.EngineHandles == baseline.Registry.EngineHandles &&
                final.Registry.ControlHandles == baseline.Registry.ControlHandles &&
                final.Registry.OutcomeHandles == baseline.Registry.OutcomeHandles &&
                final.Registry.OutcomePayloadBytes == baseline.Registry.OutcomePayloadBytes,
                AbandonedHandles: 0,
                ObservationScope:
                    "explicitly retained native handles plus whole ASP.NET process memory; admitted bytes are a lower bound, not prepared-engine retention");
        }
        finally
        {
            foreach (var outcome in retainedOutcomes)
            {
                outcome.Dispose();
            }
            foreach (var pool in pools)
            {
                pool.Dispose();
            }
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
}

public sealed record NativeRegistryPressureReport(
    int Items,
    int Concurrency,
    int Generations,
    int DelayedOutcomes,
    int SourceBytes,
    int StylesheetBytes,
    long AggregateAdmittedEngineInputBytes,
    string SemanticSentinel,
    NativeRegistryHighWater LegitimateHighWater,
    IReadOnlyList<NativeRegistryPressureCheckpoint> Checkpoints,
    IReadOnlyList<NativeRegistrySettlementSample> Settlement,
    bool LogicalRegistryReturnedToBaseline,
    int AbandonedHandles,
    string ObservationScope);

public sealed record NativeRegistryHighWater(
    ulong EngineHandles,
    ulong ControlHandles,
    ulong OutcomeHandles,
    ulong OutcomePayloadBytes);

public sealed record NativeRegistryPressureCheckpoint(
    string Phase,
    double ElapsedMilliseconds,
    NativeRegistryObservation Registry,
    long WorkingSetBytes,
    long PrivateMemoryBytes,
    long ManagedHeapBytes);

public sealed record NativeRegistrySettlementSample(
    int MillisecondsAfterRelease,
    NativeRegistryPressureCheckpoint Checkpoint);
