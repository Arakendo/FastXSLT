using System.Diagnostics;
using System.Globalization;
using System.Text;

internal static class CreationPlacementProbe
{
    private const int Warmups = 8;
    private const int Samples = 32;

    internal static async Task<object> RunAsync(string root, int order = 0)
    {
        if (order is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(order));
        NativeFastXsltClient.ConfigureRegistryPolicy(NativeRegistryPolicy.Unlimited);
        var baseline = NativeFastXsltClient.ObserveRegistry();
        var worker = Path.Combine(root, "target", "release",
            OperatingSystem.IsWindows() ? "fastxslt-worker.exe" : "fastxslt-worker");
        var directory = Path.Combine(root, "vendor", "xslt30-test", "tests", "expr", "for");
        var style = await File.ReadAllBytesAsync(Path.Combine(directory, "for-004.xsl"));
        var pinned = await File.ReadAllBytesAsync(Path.Combine(directory, "for03.xml"));
        var fixtures = new List<Fixture> { new("pinned-for-004", pinned, Output("36.02")) };
        foreach (var items in new[] { 5, 50, 500 })
        {
            var xml = new StringBuilder("<?xml version=\"1.0\"?><order>");
            for (var i = 0; i < items; i++) xml.Append("<order-item price=\"1.00\" qty=\"1\"/>");
            xml.Append("</order>");
            fixtures.Add(new($"items-{items}", Encoding.UTF8.GetBytes(xml.ToString()),
                Output(items.ToString(CultureInfo.InvariantCulture) + ".00")));
        }
        foreach (var fixture in fixtures) await PositiveParity(worker, style, fixture);
        foreach (var source in new[] { "<order>", "<!DOCTYPE order><order/>" })
            await FailureParity(worker, style, Encoding.UTF8.GetBytes(source));
        CheckBaseline(baseline);

        var report = new List<object>();
        for (var group = 0; group < fixtures.Count; group++)
        {
            var fixture = fixtures[(group + order) % fixtures.Count];
            var native = new List<NativeSample>();
            var isolated = new List<IsolatedSample>();
            for (var index = 0; index < Warmups + Samples; index++)
            {
                for (var offset = 0; offset < 2; offset++)
                {
                    if ((index + group + order + offset) % 2 == 0)
                    {
                        var sample = Native(style, fixture);
                        if (index >= Warmups) native.Add(sample);
                    }
                    else
                    {
                        var sample = await Isolated(worker, style, fixture);
                        if (index >= Warmups) isolated.Add(sample);
                    }
                    CheckBaseline(baseline);
                }
            }
            report.Add(new
            {
                fixture.Name, SourceBytes = fixture.Source.Length, StylesheetBytes = style.Length,
                Native = new
                {
                    AbiCheckUs = Summary(native.Select(s => s.Timing.AbiCheckMicroseconds)),
                    IdentityEncodingUs = Summary(native.Select(s => s.Timing.IdentityEncodingMicroseconds)),
                    CombinedNativeCreateUs = Summary(native.Select(s => s.Timing.CombinedNativeCreateMicroseconds)),
                    OutcomeOwnershipUs = Summary(native.Select(s => s.Timing.OutcomeOwnershipMicroseconds)),
                    CreationUs = Summary(native.Select(s => s.Timing.TotalCreationMicroseconds)),
                    FirstTransformAndValidationUs = Summary(native.Select(s => s.TransformUs)),
                    DisposalUs = Summary(native.Select(s => s.DisposalUs)),
                    LifecycleUs = Summary(native.Select(s => s.TotalUs)),
                    ManagedCreationAllocatedBytes = Summary(native.Select(s => (double)s.Timing.ManagedAllocatedBytes)),
                    SuppliedCopyBytes = native[0].Timing.SuppliedCopyBytes,
                },
                Isolated = new
                {
                    IdentityEncodingUs = Summary(isolated.Select(s => s.Timing.IdentityEncodingMicroseconds)),
                    ProcessLaunchReturnUs = Summary(isolated.Select(s => s.Timing.ProcessLaunchReturnMicroseconds)),
                    RequestWriteUs = Summary(isolated.Select(s => s.Timing.RequestWriteMicroseconds)),
                    RequestFlushUs = Summary(isolated.Select(s => s.Timing.RequestFlushMicroseconds)),
                    ReadinessWaitUs = Summary(isolated.Select(s => s.Timing.ReadinessWaitMicroseconds)),
                    CreationUs = Summary(isolated.Select(s => s.Timing.TotalCreationMicroseconds)),
                    FirstTransformAndValidationUs = Summary(isolated.Select(s => s.TransformUs)),
                    DisposalUs = Summary(isolated.Select(s => s.DisposalUs)),
                    LifecycleUs = Summary(isolated.Select(s => s.TotalUs)),
                    EncodedInitializationBytes = isolated[0].Timing.EncodedInitializationBytes,
                },
            });
        }
        return new
        {
            Runtime = Environment.Version.ToString(), Order = order, Samples, Warmups,
            Scope = "fresh-engine compile+prepare; one transform+validation; disposal; supplied bytes",
            PublicationEligible = false, NativeRegistryReturnedToBaseline = true,
            PositiveParityCreations = 16, FailureParityCreations = 8,
            TimedExactTransforms = Samples * fixtures.Count * 2, Measurements = report,
        };
    }

    private static NativeSample Native(byte[] style, Fixture fixture)
    {
        var start = Stopwatch.GetTimestamp();
        var (client, timing) = NativeFastXsltClient.CreateMeasured(SourceId(fixture), fixture.Source, StyleId, style);
        try
        {
            var phase = Stopwatch.GetTimestamp();
            Validate(client.Transform("urn:ar0027:creation:request"), fixture.Expected);
            var transform = Stopwatch.GetElapsedTime(phase).TotalMicroseconds;
            phase = Stopwatch.GetTimestamp();
            client.Dispose();
            var disposal = Stopwatch.GetElapsedTime(phase).TotalMicroseconds;
            return new(timing, transform, disposal, Stopwatch.GetElapsedTime(start).TotalMicroseconds);
        }
        finally { client.Dispose(); }
    }

    private static async Task<IsolatedSample> Isolated(string worker, byte[] style, Fixture fixture)
    {
        var start = Stopwatch.GetTimestamp();
        var (client, timing) = await FastXsltWorkerClient.StartMeasuredAsync(worker, SourceId(fixture), fixture.Source, StyleId, style);
        try
        {
            var phase = Stopwatch.GetTimestamp();
            Validate(await client.TransformAsync("urn:ar0027:creation:request"), fixture.Expected);
            var transform = Stopwatch.GetElapsedTime(phase).TotalMicroseconds;
            phase = Stopwatch.GetTimestamp();
            client.Dispose();
            var disposal = Stopwatch.GetElapsedTime(phase).TotalMicroseconds;
            return new(timing, transform, disposal, Stopwatch.GetElapsedTime(start).TotalMicroseconds);
        }
        finally { client.Dispose(); }
    }

    private static async Task PositiveParity(string worker, byte[] style, Fixture fixture)
    {
        using (var client = NativeFastXsltClient.Create(SourceId(fixture), fixture.Source, StyleId, style))
            Validate(client.Transform("parity"), fixture.Expected);
        using (var client = (await FastXsltWorkerClient.StartAsync(worker, SourceId(fixture), fixture.Source, StyleId, style)))
            Validate(await client.TransformAsync("parity"), fixture.Expected);
        Native(style, fixture);
        await Isolated(worker, style, fixture);
    }

    private static async Task FailureParity(string worker, byte[] style, byte[] source)
    {
        const string id = "urn:ar0027:creation:invalid-source";
        var normal = await Capture(() => { using var c = NativeFastXsltClient.Create(id, source, StyleId, style); return Task.CompletedTask; });
        var measured = await Capture(() => { using var c = NativeFastXsltClient.CreateMeasured(id, source, StyleId, style).Client; return Task.CompletedTask; });
        var isolated = await Capture(async () => { using var c = await FastXsltWorkerClient.StartAsync(worker, id, source, StyleId, style); });
        var observed = await Capture(async () => { using var c = (await FastXsltWorkerClient.StartMeasuredAsync(worker, id, source, StyleId, style)).Client; });
        if (normal != measured || normal != isolated || normal != observed)
            throw new InvalidDataException("Creation diagnostic parity failed.");
    }

    private static async Task<Failure> Capture(Func<Task> action)
    {
        try { await action(); }
        catch (NativeFastXsltException e) { return new(e.Code, e.Category, e.RequestId, e.Location, e.Detail); }
        catch (FastXsltWorkerException e) { return new(e.Code, e.Category, e.RequestId, e.Location, e.Detail); }
        throw new InvalidOperationException("Invalid input unexpectedly created an engine.");
    }

    private static object Summary(IEnumerable<double> values)
    {
        var sorted = values.Order().ToArray();
        return new { Median = (sorted[15] + sorted[16]) / 2, P95 = sorted[30], Minimum = sorted[0], Maximum = sorted[^1] };
    }
    private static void CheckBaseline(NativeRegistryObservation baseline)
    {
        if (NativeFastXsltClient.ObserveRegistry() != baseline)
            throw new InvalidDataException("Native creation retained registry state after release.");
    }
    private static void Validate(string actual, string expected)
    {
        if (!StringComparer.Ordinal.Equals(actual, expected)) throw new InvalidDataException("Creation transform output mismatch.");
    }
    private static string Output(string value) => $"<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>{value}</out>";
    private static string SourceId(Fixture fixture) => $"urn:ar0027:creation:{fixture.Name}:source";
    private const string StyleId = "urn:ar0027:creation:stylesheet";
    private sealed record Fixture(string Name, byte[] Source, string Expected);
    private sealed record NativeSample(NativeCreationTiming Timing, double TransformUs, double DisposalUs, double TotalUs);
    private sealed record IsolatedSample(IsolatedCreationTiming Timing, double TransformUs, double DisposalUs, double TotalUs);
    private sealed record Failure(string Code, string Category, string? Request, FastXsltDiagnosticLocation? Location, string Detail);
}
