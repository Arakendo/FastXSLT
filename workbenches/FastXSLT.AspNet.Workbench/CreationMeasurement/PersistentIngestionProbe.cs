using System.Diagnostics;
using System.Globalization;
using System.Text;

internal static class PersistentIngestionProbe
{
    internal static async Task<object> RunAsync(string root, int jobs, int order)
    {
        if (jobs is < 128 or > 5000) throw new ArgumentOutOfRangeException(nameof(jobs));
        if (order is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(order));
        var directory = Path.Combine(root, "vendor", "xslt30-test", "tests", "expr", "for");
        var style = await File.ReadAllBytesAsync(Path.Combine(directory, "for-004.xsl"));
        var fixtures = new List<Fixture> { new("pinned", await File.ReadAllBytesAsync(
            Path.Combine(directory, "for03.xml")), Output("36.02")) };
        foreach (var items in new[] { 5, 50, 500 })
            fixtures.Add(new($"items-{items}", Encoding.UTF8.GetBytes("<?xml version=\"1.0\"?><order>" +
                string.Concat(Enumerable.Repeat("<order-item price=\"1.00\" qty=\"1\"/>", items)) + "</order>"),
                Output(items.ToString(CultureInfo.InvariantCulture) + ".00")));
        return await RunFixturesAsync(root, jobs, order, style, fixtures, "decimal-items");
    }

    internal static Task<object> RunShapesAsync(string root, int jobs, int order)
    {
        if (jobs is < 128 or > 5000) throw new ArgumentOutOfRangeException(nameof(jobs));
        if (order is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(order));
        return RunFixturesAsync(root, jobs, order, ShapeIngestionFixtures.Style,
            ShapeIngestionFixtures.Create(), "copy-shapes");
    }

    private static async Task<object> RunFixturesAsync(string root, int jobs, int order,
        byte[] style, IReadOnlyList<Fixture> fixtures, string family)
    {
        NativeFastXsltClient.ConfigureRegistryPolicy(NativeRegistryPolicy.Unlimited);
        var baseline = NativeFastXsltClient.ObserveRegistry();
        var worker = Path.Combine(root, "target", "release", OperatingSystem.IsWindows() ?
            "fastxslt-worker.exe" : "fastxslt-worker");
        await VerifyLostWorker(worker, style, fixtures[0]);
        using var isolated = await FastXsltWorkerClient.StartAsync(worker, "urn:ar0027:seed", fixtures[0].Source, StyleId, style);
        var pid = isolated.ProcessId;
        NativeFastXsltClient? native = NativeFastXsltClient.Create("urn:ar0027:seed", fixtures[0].Source, StyleId, style);
        var samples = fixtures.Select(_ => new List<Sample>()).ToArray();
        var checks = 0;
        NativeFastXsltException? depthFailure = null;
        try
        {
            // Failure before publication must leave the previously admitted engine usable.
            var rejectedSources = new List<string> { "<order>", "<!DOCTYPE order><order/>" };
            if (family == "copy-shapes") rejectedSources.Add(
                string.Concat(Enumerable.Repeat("<n>", 256)) + "leaf" +
                string.Concat(Enumerable.Repeat("</n>", 256)));
            foreach (var invalid in rejectedSources)
            {
                var bytes = Encoding.UTF8.GetBytes(invalid);
                NativeFastXsltException nativeFailure;
                try { using var unexpected = NativeFastXsltClient.Create("urn:ar0027:invalid", bytes, StyleId, style);
                    throw new InvalidDataException("Invalid source was accepted."); }
                catch (NativeFastXsltException failure) { nativeFailure = failure; }
                if (family == "copy-shapes" && invalid == rejectedSources[^1]) depthFailure = nativeFailure;
                try { await isolated.ReinitializeMeasuredAsync("urn:ar0027:invalid", bytes, StyleId, style);
                    throw new InvalidDataException("Invalid source was accepted."); }
                catch (FastXsltWorkerException failure)
                {
                    if (failure.Code != nativeFailure.Code || failure.Category != nativeFailure.Category ||
                        failure.RequestId != nativeFailure.RequestId || failure.Location != nativeFailure.Location ||
                        failure.Detail != nativeFailure.Detail) throw new InvalidDataException("Failure parity mismatch.");
                }
                Validate(native.Transform("after-failure"), fixtures[0].Expected);
                Validate(await isolated.TransformAsync("after-failure"), fixtures[0].Expected);
                checks += 2;
            }
            // Reject an oversized field before framing; the same worker must remain usable.
            try { await isolated.ReinitializeMeasuredAsync("urn:ar0027:oversized", new byte[1_048_577], StyleId, style);
                throw new InvalidDataException("Oversized source was accepted."); }
            catch (ArgumentOutOfRangeException) { }
            Validate(await isolated.TransformAsync("after-field-limit"), fixtures[0].Expected);
            checks++;

            for (var index = -32; index < jobs; index++)
            {
                var group = ((index + 32 + order) % fixtures.Count);
                var fixture = fixtures[group];
                // Distinct logical origins; acquisition and identity assembly are outside timing.
                var identity = $"urn:ar0027:ingest:{order}:{index}:{group}";
                var request = $"urn:ar0027:request:{order}:{index}";
                double nativeCreate = 0, nativeRetire = 0, nativeTotal = 0;
                PersistentInitializationTiming? timing = null;
                double isolatedTotal = 0;
                // A repeated even-sized fixture cycle must not pin each shape to one first lane.
                var nativeFirst = (index + order + (index + 32) / fixtures.Count) % 2 == 0;
                for (var lane = 0; lane < 2; lane++)
                {
                    var start = Stopwatch.GetTimestamp();
                    if ((lane == 0) == nativeFirst)
                    {
                        var replacement = NativeFastXsltClient.CreateMeasured(identity, fixture.Source, StyleId, style);
                        nativeCreate = replacement.Timing.TotalCreationMicroseconds;
                        var retired = Stopwatch.GetTimestamp();
                        native.Dispose();
                        nativeRetire = Stopwatch.GetElapsedTime(retired).TotalMicroseconds;
                        native = replacement.Client;
                        Validate(native.Transform(request), fixture.Expected, fixture.Name);
                        nativeTotal = Stopwatch.GetElapsedTime(start).TotalMicroseconds;
                    }
                    else
                    {
                        timing = await isolated.ReinitializeMeasuredAsync(identity, fixture.Source, StyleId, style);
                        Validate(await isolated.TransformAsync(request), fixture.Expected, fixture.Name);
                        isolatedTotal = Stopwatch.GetElapsedTime(start).TotalMicroseconds;
                    }
                }
                if (index >= 0) samples[group].Add(new(nativeCreate, nativeRetire, nativeTotal, timing!, isolatedTotal, nativeFirst));
                if (isolated.ProcessId != pid) throw new InvalidDataException("Worker changed during ingestion.");
            }
        }
        finally { native?.Dispose(); isolated.Dispose(); }
        if (NativeFastXsltClient.ObserveRegistry() != baseline) throw new InvalidDataException("Registry retained state.");
        return new
        {
            JobsPerLane = jobs, Order = order, WarmupsPerLane = 32, Runtime = Environment.Version.ToString(),
            Family = family, PreparationCapacity = "production-growth",
            RejectedDepthControl = family == "copy-shapes" ? (int?)256 : null,
            DepthDiagnostic = depthFailure is null ? null : new { depthFailure.Code, depthFailure.Category, depthFailure.Location },
            StructuralLimitMisclassifiedAsInvalid = depthFailure?.Category == "invalid",
            TimedExactResults = jobs * 2, RecoveryExactResults = checks,
            LostWorkerRetiredWithoutRetry = true,
            WorkerProcessUnchanged = true, NativeRegistryReturnedToBaseline = true, PublicationEligible = false,
            Scope = "sequential distinct-origin queue; full recompile+prepare; prior generation retained until successful replacement",
            Measurements = fixtures.Select((fixture, i) => new
            {
                fixture.Name, SourceBytes = fixture.Source.Length,
                ExpectedResultUtf8Bytes = Encoding.UTF8.GetByteCount(fixture.Expected), Samples = samples[i].Count,
                PreviousShape = fixtures[(i + fixtures.Count - 1) % fixtures.Count].Name,
                NativeFirstSamples = samples[i].Count(s => s.NativeFirst),
                IsolatedFirstSamples = samples[i].Count(s => !s.NativeFirst),
                NativeCreationUs = Summary(samples[i].Select(s => s.NativeCreate)),
                NativeOldEngineRetirementUs = Summary(samples[i].Select(s => s.NativeRetire)),
                NativeThroughResultUs = Summary(samples[i].Select(s => s.NativeTotal)),
                IsolatedInitializationUs = Summary(samples[i].Select(s => s.Isolated.TotalUs)),
                IsolatedWriteUs = Summary(samples[i].Select(s => s.Isolated.WriteUs)),
                IsolatedReadyUs = Summary(samples[i].Select(s => s.Isolated.ReadinessUs)),
                IsolatedThroughResultUs = Summary(samples[i].Select(s => s.IsolatedTotal)),
            }).ToArray(),
        };
    }
    private static async Task VerifyLostWorker(string worker, byte[] style, Fixture fixture)
    {
        using (var lost = await FastXsltWorkerClient.StartAsync(worker, "urn:ar0027:lost", fixture.Source, StyleId, style))
        {
            lost.TerminateForExperiment();
            try { await lost.ReinitializeMeasuredAsync("urn:ar0027:after-loss", fixture.Source, StyleId, style);
                throw new InvalidDataException("Lost worker accepted initialization."); }
            catch (IOException) { }
        }
        // Recovery is an explicit fresh worker, never an implicit retry.
        using var fresh = await FastXsltWorkerClient.StartAsync(worker, "urn:ar0027:recovery", fixture.Source, StyleId, style);
        Validate(await fresh.TransformAsync("after-loss-recovery"), fixture.Expected);
    }
    private static object Summary(IEnumerable<double> values)
    {
        var sorted = values.Order().ToArray();
        var n = sorted.Length;
        return new { Median = (sorted[(n - 1) / 2] + sorted[n / 2]) / 2,
            P95 = sorted[(int)Math.Ceiling(n * .95) - 1], Minimum = sorted[0], Maximum = sorted[^1] };
    }
    private static string Output(string value) => $"<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>{value}</out>";
    private static void Validate(string result, string expected, string? fixture = null)
    {
        if (StringComparer.Ordinal.Equals(result, expected)) return;
        var first = 0;
        while (first < Math.Min(result.Length, expected.Length) && result[first] == expected[first]) first++;
        throw new InvalidDataException($"Exact result mismatch in {fixture ?? "recovery"} at UTF-16 offset {first}; " +
            $"expected/actual lengths {expected.Length}/{result.Length}.");
    }
    private const string StyleId = "urn:ar0027:creation:stylesheet";
    internal sealed record Fixture(string Name, byte[] Source, string Expected);
    private sealed record Sample(double NativeCreate, double NativeRetire, double NativeTotal,
        PersistentInitializationTiming Isolated, double IsolatedTotal, bool NativeFirst);
}
