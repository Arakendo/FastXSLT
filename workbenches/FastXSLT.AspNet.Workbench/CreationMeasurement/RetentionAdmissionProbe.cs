using System.Text;

// Private production-growth ownership control, not capacity-candidate adoption.
internal static class RetentionAdmissionProbe
{
    private const string StyleId = "urn:ar0027:retention:style";
    private static readonly byte[] Style = Encoding.UTF8.GetBytes("<xsl:stylesheet version=\"1.0\" xmlns:xsl=\"http://www.w3.org/1999/XSL/Transform\"><xsl:template match=\"/\"><xsl:copy-of select=\"/root\"/></xsl:template></xsl:stylesheet>");

    internal static async Task<object> RunAsync(string root)
    {
        // Fresh-process experimental values, not defaults or a hard memory cap.
        NativeFastXsltClient.ConfigureRegistryPolicy(new(2, 0, 3, 1_048_576, ulong.MaxValue, ulong.MaxValue));
        var baseline = NativeFastXsltClient.ObserveRegistry();
        if (baseline != new NativeRegistryObservation(0, 0, 0, 0)) throw new InvalidDataException("Probe requires an empty registry.");
        var small = Fixture(5, 8);
        var large = Fixture(500, 9);
        var medium = Fixture(50, 10);
        var native = Native(small, large, medium);
        var worker = Path.Combine(root, "target", "release", OperatingSystem.IsWindows() ? "fastxslt-worker.exe" : "fastxslt-worker");
        var isolated = await IsolatedAsync(worker, small, large);
        if (NativeFastXsltClient.ObserveRegistry() != baseline) throw new InvalidDataException("Native ownership did not drain.");
        return new { Runtime = Environment.Version.ToString(), Native = native, Isolated = isolated,
            ProductionGrowthOnly = true, CandidateAdopted = false, PublicationEligible = false,
            Scope = "quiescent exact native counts/payload; isolated host-copy ownership; no engine-byte/RSS/peak measurement" };
    }

    private static object Native(Input small, Input large, Input medium)
    {
        var checks = new Checks();
        var checkpoints = new List<object>();
        void Observe(string stage, ulong engines, ulong outcomes, ulong bytes)
        {
            var actual = NativeFastXsltClient.ObserveRegistry();
            if (actual != new NativeRegistryObservation(engines, 0, outcomes, bytes))
                throw new InvalidDataException($"Registry ownership mismatch at {stage}.");
            checkpoints.Add(new { Stage = stage, Observation = actual });
        }
        using var old = NativeFastXsltClient.Create(small.Identity, small.Bytes, StyleId, Style);
        using var oldResult = old.TransformRetained("retained-old");
        var smallBytes = checked((ulong)Encoding.UTF8.GetByteCount(small.Expected));
        var largeBytes = checked((ulong)Encoding.UTF8.GetByteCount(large.Expected));
        Observe("old-and-result", 1, 1, smallBytes);
        using var current = NativeFastXsltClient.Create(large.Identity, large.Bytes, StyleId, Style);
        using var currentResult = current.TransformRetained("retained-new");
        Observe("overlapping-generations-and-results", 2, 2, smallBytes + largeBytes);
        try
        {
            using var unexpected = NativeFastXsltClient.Create(medium.Identity, medium.Bytes, StyleId, Style);
            throw new InvalidDataException("Third engine bypassed quota.");
        }
        catch (NativeFastXsltException failure) when (failure.Code == "FXFFI0102" && failure.Category == "resource-exhausted") { }
        Observe("denied-third-engine-no-extra-outcome", 2, 2, smallBytes + largeBytes);
        try
        {
            using var unexpected = NativeFastXsltClient.Create("urn:ar0027:invalid", Encoding.UTF8.GetBytes("<root>"), StyleId, Style);
            throw new InvalidDataException("Malformed replacement was accepted.");
        }
        catch (NativeFastXsltException failure) when (failure.Category == "invalid") { }
        Observe("failed-initialization-drained", 2, 2, smallBytes + largeBytes);
        try { current.TransformWithInvocationPolicy("cancel-retention", true, ulong.MaxValue); throw new InvalidDataException("Cancelled work succeeded."); }
        catch (NativeFastXsltException failure) when (failure.Code == "FXCT0001" && failure.Category == "cancelled") { }
        Observe("cancelled-invocation-drained", 2, 2, smallBytes + largeBytes);
        checks.Match(old.Transform("old-reuse"), small.Expected);
        checks.Match(current.Transform("new-reuse"), large.Expected);
        old.Dispose();
        Observe("old-engine-released-result-survives", 1, 2, smallBytes + largeBytes);
        checks.Match(oldResult.ReadResult(), small.Expected);
        using (var recovered = NativeFastXsltClient.Create(medium.Identity, medium.Bytes, StyleId, Style))
        {
            Observe("released-slot-readmitted", 2, 2, smallBytes + largeBytes);
            checks.Match(recovered.Transform("slot-recovery"), medium.Expected);
        }
        current.Dispose();
        Observe("all-engines-released-results-survive", 0, 2, smallBytes + largeBytes);
        checks.Match(oldResult.ReadResult(), small.Expected);
        checks.Match(currentResult.ReadResult(), large.Expected);
        oldResult.Dispose();
        Observe("old-result-released", 0, 1, largeBytes);
        currentResult.Dispose();
        Observe("fully-drained", 0, 0, 0);
        return new { ExactResults = checks.Count, EngineLimit = 2, OutcomeLimit = 3, OutcomeByteLimit = 1_048_576,
            KnownEngineCapacityLimitOptedOut = true, AccountedTotalLimitOptedOut = true, Checkpoints = checkpoints };
    }

    private static async Task<object> IsolatedAsync(string worker, Input small, Input large)
    {
        var checks = new Checks();
        using var old = await FastXsltWorkerClient.StartAsync(worker, small.Identity, small.Bytes, StyleId, Style);
        using var current = await FastXsltWorkerClient.StartAsync(worker, large.Identity, large.Bytes, StyleId, Style);
        var oldPid = old.ProcessId;
        var currentPid = current.ProcessId;
        if (oldPid == currentPid) throw new InvalidDataException("Worker generations aliased.");
        var oldResult = await old.TransformAsync("isolated-old");
        var currentResult = await current.TransformAsync("isolated-new");
        checks.Match(oldResult, small.Expected);
        checks.Match(currentResult, large.Expected);
        try
        {
            await current.ReinitializeMeasuredAsync("urn:ar0027:invalid", Encoding.UTF8.GetBytes("<root>"), StyleId, Style);
            throw new InvalidDataException("Malformed isolated replacement was accepted.");
        }
        catch (FastXsltWorkerException failure) when (failure.Category == "invalid") { }
        checks.Match(await current.TransformAsync("isolated-after-failure"), large.Expected);
        if (current.ProcessId != currentPid || old.ProcessId != oldPid) throw new InvalidDataException("Implicit worker replacement.");
        old.Dispose();
        checks.Match(oldResult, small.Expected);
        checks.Match(await current.TransformAsync("isolated-after-old-disposal"), large.Expected);
        current.Dispose();
        checks.Match(oldResult, small.Expected);
        checks.Match(currentResult, large.Expected);
        return new { ExactResults = checks.Count, IndependentWorkerGenerations = 2,
            FailedInitializationPreservesGeneration = true, NoImplicitReplacementOrRetry = true,
            ResultCopiesSurviveDisposal = true, ResultOwnership = "managed strings already transferred; not worker-retained outcomes" };
    }

    private static Input Fixture(int items, int job)
    {
        var body = $"<root job=\"{job}\">" + string.Concat(Enumerable.Range(0, items).Select(index =>
            $"<item n=\"{index}\">{new string('x', 512)}&amp;&lt;</item>")) + "</root>";
        return new($"urn:ar0027:copy:{items}:{job}", Encoding.UTF8.GetBytes("<?xml version=\"1.0\"?>" + body),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>" + body);
    }
    private sealed record Input(string Identity, byte[] Bytes, string Expected);
    private sealed class Checks
    {
        internal int Count { get; private set; }
        internal void Match(string actual, string expected)
        {
            if (!StringComparer.Ordinal.Equals(actual, expected)) throw new InvalidDataException("Exact result mismatch.");
            Count++;
        }
    }
}
