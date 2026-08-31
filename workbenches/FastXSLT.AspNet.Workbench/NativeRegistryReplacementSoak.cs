using System.Diagnostics;
using System.Text;

public static class NativeRegistryReplacementSoak
{
    public static async Task<NativeRegistryReplacementSoakReport> RunAsync(
        byte[] stylesheet,
        int concurrency,
        int replacements,
        int retainedOldGenerations,
        int requestsPerGeneration)
    {
        concurrency = Math.Clamp(concurrency, 1, 32);
        replacements = Math.Clamp(replacements, 1, 128);
        retainedOldGenerations = Math.Clamp(retainedOldGenerations, 1, 3);
        requestsPerGeneration = Math.Clamp(requestsPerGeneration, 1, 256);

        var baseline = Observe("baseline", 0);
        var experiment = Stopwatch.StartNew();
        var checkpoints = new List<NativeRegistryPressureCheckpoint>(replacements + 3)
        {
            baseline
        };
        var replacementLatencies = new double[replacements];
        var transformLatencies = new double[checked(replacements * requestsPerGeneration)];
        var held = new Queue<HeldGeneration>();
        var currentItems = 500;
        var host = NativeFastXsltGenerationHost.Create(
            "registry-soak-g0",
            "urn:fastxslt:registry-soak:source:g0",
            BuildSource(currentItems),
            "urn:fastxslt:registry-soak:stylesheet:g0",
            stylesheet,
            concurrency);

        try
        {
            checkpoints.Add(Observe("initial-generation-prepared", experiment.Elapsed.TotalMilliseconds));
            for (var replacement = 1; replacement <= replacements; replacement++)
            {
                var oldLease = host.AcquireCurrent();
                held.Enqueue(new HeldGeneration(oldLease, Expected(currentItems)));
                while (held.Count > retainedOldGenerations)
                {
                    held.Dequeue().Lease.Dispose();
                }

                var nextItems = 500 + replacement % 2;
                var generationIdentity = $"registry-soak-g{replacement}";
                var replacementStarted = Stopwatch.GetTimestamp();
                var retired = host.Replace(
                    generationIdentity,
                    $"urn:fastxslt:registry-soak:source:g{replacement}",
                    BuildSource(nextItems),
                    $"urn:fastxslt:registry-soak:stylesheet:g{replacement}",
                    stylesheet,
                    concurrency);
                replacementLatencies[replacement - 1] =
                    Stopwatch.GetElapsedTime(replacementStarted).TotalMicroseconds;
                if (!StringComparer.Ordinal.Equals(retired, oldLease.Identity))
                {
                    throw new InvalidOperationException(
                        "Sustained replacement retired a different generation than the held lease.");
                }

                checkpoints.Add(Observe(
                    $"replacement-{replacement}-promoted",
                    experiment.Elapsed.TotalMilliseconds));

                var expected = Expected(nextItems);
                await Parallel.ForEachAsync(
                    Enumerable.Range(0, requestsPerGeneration),
                    new ParallelOptions { MaxDegreeOfParallelism = concurrency },
                    async (request, _) =>
                    {
                        var started = Stopwatch.GetTimestamp();
                        var result = await host.TransformAsync(
                            $"registry-soak-g{replacement}-request-{request}");
                        transformLatencies[(replacement - 1) * requestsPerGeneration + request] =
                            Stopwatch.GetElapsedTime(started).TotalMicroseconds;
                        if (!StringComparer.Ordinal.Equals(result.GenerationIdentity, generationIdentity) ||
                            !StringComparer.Ordinal.Equals(result.Result, expected))
                        {
                            throw new InvalidOperationException(
                                "Sustained replacement changed new-generation identity or semantics.");
                        }
                    });

                var heldGeneration = held.Last();
                var oldResult = await heldGeneration.Lease.Pool.TransformAsync(
                    $"registry-soak-retired-{replacement}");
                if (!StringComparer.Ordinal.Equals(oldResult, heldGeneration.Expected))
                {
                    throw new InvalidOperationException(
                        "Sustained replacement changed a retained old generation's semantics.");
                }

                currentItems = nextItems;
            }

            while (held.Count > 0)
            {
                held.Dequeue().Lease.Dispose();
            }
            checkpoints.Add(Observe(
                "old-generations-drained",
                experiment.Elapsed.TotalMilliseconds));
            host.Dispose();
            checkpoints.Add(Observe(
                "all-experiment-generations-released",
                experiment.Elapsed.TotalMilliseconds));

            Array.Sort(replacementLatencies);
            Array.Sort(transformLatencies);
            var highWater = new NativeRegistryHighWater(
                checkpoints.Max(value => value.Registry.EngineHandles),
                checkpoints.Max(value => value.Registry.ControlHandles),
                checkpoints.Max(value => value.Registry.OutcomeHandles),
                checkpoints.Max(value => value.Registry.OutcomePayloadBytes));
            var final = checkpoints[^1].Registry;
            return new NativeRegistryReplacementSoakReport(
                concurrency,
                replacements,
                retainedOldGenerations,
                requestsPerGeneration,
                replacementLatencies.Length,
                transformLatencies.Length,
                Percentile(replacementLatencies, 0.50),
                Percentile(replacementLatencies, 0.95),
                Percentile(replacementLatencies, 0.99),
                Percentile(transformLatencies, 0.50),
                Percentile(transformLatencies, 0.95),
                Percentile(transformLatencies, 0.99),
                highWater,
                checkpoints,
                final == baseline.Registry,
                "sustained explicit generation replacement with bounded old leases, new-generation requests, and old-generation semantic sentinels");
        }
        finally
        {
            while (held.Count > 0)
            {
                held.Dequeue().Lease.Dispose();
            }
            host.Dispose();
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

    private static double Percentile(double[] sorted, double percentile)
    {
        var index = (int)Math.Ceiling(percentile * sorted.Length) - 1;
        return sorted[Math.Clamp(index, 0, sorted.Length - 1)];
    }

    private static string Expected(int items) =>
        $"<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>{items}.00</out>";

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

    private sealed record HeldGeneration(
        NativeFastXsltGenerationHost.GenerationLease Lease,
        string Expected);
}

public sealed record NativeRegistryReplacementSoakReport(
    int Concurrency,
    int Replacements,
    int RetainedOldGenerations,
    int RequestsPerGeneration,
    int ReplacementSamples,
    int TransformSamples,
    double ReplacementP50Microseconds,
    double ReplacementP95Microseconds,
    double ReplacementP99Microseconds,
    double TransformP50Microseconds,
    double TransformP95Microseconds,
    double TransformP99Microseconds,
    NativeRegistryHighWater LegitimateHighWater,
    IReadOnlyList<NativeRegistryPressureCheckpoint> Checkpoints,
    bool LogicalRegistryReturnedToBaseline,
    string ObservationScope);
