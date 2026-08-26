using System.Diagnostics;

var builder = WebApplication.CreateBuilder(args);
var repositoryRoot = FindRepositoryRoot(builder.Environment.ContentRootPath);
var executableName = OperatingSystem.IsWindows() ? "fastxslt-worker.exe" : "fastxslt-worker";
var workerPath = Path.Combine(repositoryRoot, "target", "release", executableName);
var sourcePath = Path.Combine(
    repositoryRoot, "vendor", "xslt30-test", "tests", "expr", "for", "for03.xml");
var stylesheetPath = Path.Combine(
    repositoryRoot, "vendor", "xslt30-test", "tests", "expr", "for", "for-004.xsl");
var dotNetStylesheetPath = Path.Combine(
    repositoryRoot, "workbenches", "FastXSLT.AspNet.Workbench", "fixtures",
    "for-004-equivalent-xslt1.xsl");
var dotNetStylesheet = await File.ReadAllBytesAsync(dotNetStylesheetPath);

var source = await File.ReadAllBytesAsync(sourcePath);
var stylesheet = await File.ReadAllBytesAsync(stylesheetPath);

var worker = await FastXsltWorkerClient.StartAsync(
    workerPath,
    "urn:w3c:xslt30:for-004:source",
    source,
    "urn:w3c:xslt30:for-004:stylesheet",
    stylesheet);
var dotNetXslt1 = DotNetXslt1Baseline.Create(
    source,
    dotNetStylesheet);
#if SAXONCS_LOCAL
var saxonCs = SaxonCsBaseline.Create(source, stylesheet);
#endif
var exactStylesheetProbe = DotNetXslt1Baseline.ProbeExactStylesheet(source, stylesheet);
builder.Services.AddSingleton(worker);
builder.Services.AddSingleton(dotNetXslt1);
#if SAXONCS_LOCAL
builder.Services.AddSingleton(saxonCs);
#endif

var app = builder.Build();
var tieredBenchmarkGate = new SemaphoreSlim(1, 1);
var operationalExperimentGate = new SemaphoreSlim(1, 1);
app.MapGet("/health", () => Results.Ok(new
{
    status = "ready",
    mode = "isolated-persistent-worker",
    maximumInFlight = 1,
    semantics = "xslt30-for-004-private-slice",
    dotNetXslt1ExactStylesheetExecuted = exactStylesheetProbe.Executed,
    dotNetXslt1ExactStylesheetDiagnostic = exactStylesheetProbe.Detail,
#if SAXONCS_LOCAL
    saxonCsAvailable = true
#else
    saxonCsAvailable = false
#endif
}));
app.MapPost("/transform/{requestId}", async (string requestId, FastXsltWorkerClient client) =>
{
    try
    {
        var result = await client.TransformAsync(requestId);
        return Results.Text(result, "application/xml");
    }
    catch (FastXsltWorkerException failure)
    {
        return Results.Json(new
        {
            failure.Code,
            failure.Category,
            failure.RequestId,
            failure.Detail
        }, statusCode: StatusCodes.Status422UnprocessableEntity);
    }
});
app.MapPost("/transform/dotnet-xslt1", (DotNetXslt1Baseline baseline) =>
    Results.Text(baseline.Transform(), "application/xml"));
#if SAXONCS_LOCAL
app.MapPost("/transform/saxoncs", (SaxonCsBaseline baseline) =>
    Results.Text(baseline.Transform(), "application/xml"));
#endif
app.MapPost("/measure", async (int? requests, FastXsltWorkerClient client) =>
{
    var count = Math.Clamp(requests ?? 100, 1, 10_000);
    var stopwatch = Stopwatch.StartNew();
    for (var index = 0; index < count; index++)
    {
        _ = await client.TransformAsync($"measure-{index}");
    }
    stopwatch.Stop();
    return Results.Ok(new
    {
        requests = count,
        elapsedMilliseconds = stopwatch.Elapsed.TotalMilliseconds,
        transformsPerSecond = count / stopwatch.Elapsed.TotalSeconds,
        maximumInFlight = 1
    });
});
app.MapPost("/measure/dotnet-xslt1", (int? requests, DotNetXslt1Baseline baseline) =>
{
    var count = Math.Clamp(requests ?? 100, 1, 10_000);
    var stopwatch = Stopwatch.StartNew();
    for (var index = 0; index < count; index++)
    {
        _ = baseline.Transform();
    }
    stopwatch.Stop();
    return Results.Ok(new
    {
        requests = count,
        elapsedMilliseconds = stopwatch.Elapsed.TotalMilliseconds,
        transformsPerSecond = count / stopwatch.Elapsed.TotalSeconds,
        maximumInFlight = 1
    });
});
app.MapPost("/benchmark/tiers", async (int? requests, int? concurrency) =>
{
    await tieredBenchmarkGate.WaitAsync();
    try
    {
        return Results.Ok(await TieredComparison.RunAsync(
            workerPath,
            stylesheet,
            dotNetStylesheet,
            Math.Clamp(requests ?? 250, 1, 10_000),
            Math.Clamp(concurrency ?? 4, 1, 8)));
    }
    finally
    {
        tieredBenchmarkGate.Release();
    }
});
app.MapPost("/experiment/worker-recovery", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await OperationalExperiments.ExerciseWorkerRecoveryAsync(
            workerPath,
            source,
            stylesheet));
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
app.MapPost("/experiment/cooperative-cancellation", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await OperationalExperiments.ExerciseCooperativeCancellationAsync(
            workerPath,
            source,
            stylesheet));
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
app.MapPost("/experiment/active-cancellation", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await OperationalExperiments.ExerciseActiveCancellationAsync(
            workerPath,
            stylesheet));
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
app.MapPost("/experiment/natural-cancellation-races", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await OperationalExperiments.MeasureNaturalCancellationRacesAsync(
            workerPath,
            stylesheet));
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
app.MapPost("/experiment/generation-replacement", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await OperationalExperiments.ExerciseGenerationReplacementAsync(
            workerPath,
            source,
            stylesheet));
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
app.MapPost("/experiment/host-file-replacement", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await OperationalExperiments.ExerciseHostFileReplacementAsync(
            workerPath,
            Path.Combine(repositoryRoot, ".workbench"),
            stylesheet));
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
#if SAXONCS_LOCAL
app.MapPost("/measure/saxoncs", (int? requests, SaxonCsBaseline baseline) =>
{
    var count = Math.Clamp(requests ?? 100, 1, 10_000);
    var stopwatch = Stopwatch.StartNew();
    for (var index = 0; index < count; index++)
    {
        _ = baseline.Transform();
    }
    stopwatch.Stop();
    return Results.Ok(new
    {
        requests = count,
        elapsedMilliseconds = stopwatch.Elapsed.TotalMilliseconds,
        transformsPerSecond = count / stopwatch.Elapsed.TotalSeconds,
        maximumInFlight = 1
    });
});
#endif

app.Lifetime.ApplicationStopping.Register(worker.Dispose);
await app.RunAsync();

static string FindRepositoryRoot(string start)
{
    for (var current = new DirectoryInfo(start); current is not null; current = current.Parent)
    {
        if (File.Exists(Path.Combine(current.FullName, "Cargo.toml")) &&
            Directory.Exists(Path.Combine(current.FullName, "vendor", "xslt30-test")))
        {
            return current.FullName;
        }
    }
    throw new InvalidOperationException("Could not locate the FastXSLT repository root.");
}
