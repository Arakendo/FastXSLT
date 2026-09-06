using System.Diagnostics;

if (args.Contains("--native-quota-smoke", StringComparer.Ordinal))
{
    NativeFastXsltClient.ConfigureRegistryPolicy(new NativeRegistryPolicy(
        MaxEngines: ulong.MaxValue,
        MaxControls: 0,
        MaxOutcomes: ulong.MaxValue,
        MaxOutcomePayloadBytes: ulong.MaxValue,
        MaxEngineKnownCapacityBytes: ulong.MaxValue,
        MaxAccountedBytes: ulong.MaxValue));
    try
    {
        using var unexpected = NativeFastXsltClient.NativeControlHandle.Create(
            firstChargeBarrier: false);
        throw new InvalidOperationException("Zero control quota admitted a native handle.");
    }
    catch (NativeFastXsltException failure) when (
        failure.Code == "FXFFI0103" && failure.Category == "resource-exhausted")
    {
        Console.WriteLine("native-quota-smoke: FXFFI0103 resource-exhausted");
        return;
    }
}

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
var dotNetLinearStylesheetPath = Path.Combine(
    repositoryRoot, "workbenches", "FastXSLT.AspNet.Workbench", "fixtures",
    "for-004-equivalent-xslt1-linear.xsl");
var dotNetLinearStylesheet = await File.ReadAllBytesAsync(dotNetLinearStylesheetPath);

var source = await File.ReadAllBytesAsync(sourcePath);
var stylesheet = await File.ReadAllBytesAsync(stylesheetPath);

// This unpublished comparison host opts out explicitly so its historical
// pressure experiments remain comparable. Product hosts must supply their own
// count and accounted-byte envelope before creating native handles.
NativeFastXsltClient.ConfigureRegistryPolicy(NativeRegistryPolicy.Unlimited);

var worker = await FastXsltWorkerClient.StartAsync(
    workerPath,
    "urn:w3c:xslt30:for-004:source",
    source,
    "urn:w3c:xslt30:for-004:stylesheet",
    stylesheet);
var native = NativeFastXsltClient.Create(
    "urn:w3c:xslt30:for-004:source",
    source,
    "urn:w3c:xslt30:for-004:stylesheet",
    stylesheet);
var dotNetXslt1 = DotNetXslt1Baseline.Create(
    source,
    dotNetStylesheet);
#if SAXONCS_LOCAL
var saxonCs = SaxonCsBaseline.Create(source, stylesheet);
var saxonDestinationParity = SaxonCsBaseline.ProbeDestinations();
#endif
var exactStylesheetProbe = DotNetXslt1Baseline.ProbeExactStylesheet(source, stylesheet);
builder.Services.AddSingleton(worker);
builder.Services.AddSingleton(native);
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
    nativeInProcessAvailable = true,
    semantics = "xslt30-for-004-private-slice",
    dotNetXslt1ExactStylesheetExecuted = exactStylesheetProbe.Executed,
    dotNetXslt1ExactStylesheetDiagnostic = exactStylesheetProbe.Detail,
#if SAXONCS_LOCAL
    saxonCsAvailable = true,
    saxonDestinationParity
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
app.MapPost("/transform/inprocess/{requestId}", (string requestId, NativeFastXsltClient client) =>
{
    try
    {
        return Results.Text(client.Transform(requestId), "application/xml");
    }
    catch (NativeFastXsltException failure)
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
app.MapPost("/measure/inprocess", (int? requests, NativeFastXsltClient client) =>
{
    var count = Math.Clamp(requests ?? 100, 1, 10_000);
    var stopwatch = Stopwatch.StartNew();
    for (var index = 0; index < count; index++)
    {
        _ = client.Transform($"native-measure-{index}");
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
app.MapPost("/benchmark/tiers", async (int? requests, int? concurrency, int? orderSeed) =>
{
    await tieredBenchmarkGate.WaitAsync();
    try
    {
        return Results.Ok(await TieredComparison.RunAsync(
            workerPath,
            stylesheet,
            dotNetStylesheet,
            dotNetLinearStylesheet,
            Math.Clamp(requests ?? 250, 1, 10_000),
            Math.Clamp(concurrency ?? 4, 1, 8),
            orderSeed));
    }
    finally
    {
        tieredBenchmarkGate.Release();
    }
});
app.MapPost("/benchmark/best-practice-deployment", async (
    int? members,
    int? concurrency,
    int? orderSeed) =>
{
    await tieredBenchmarkGate.WaitAsync();
    try
    {
        return Results.Ok(await BestPracticeDeploymentComparison.RunAsync(
            workerPath,
            stylesheet,
            dotNetLinearStylesheet,
            Math.Clamp(members ?? 4_000, 128, 20_000),
            Math.Clamp(concurrency ?? 4, 1, 8),
            orderSeed));
    }
    finally
    {
        tieredBenchmarkGate.Release();
    }
});
app.MapPost("/benchmark/text-heavy", async (int? requests, int? concurrency) =>
{
    await tieredBenchmarkGate.WaitAsync();
    try
    {
        return Results.Ok(await TieredComparison.RunTextHeavyAsync(
            workerPath,
            Math.Clamp(requests ?? 100, 1, 10_000),
            Math.Clamp(concurrency ?? 4, 1, 8)));
    }
    finally
    {
        tieredBenchmarkGate.Release();
    }
});
app.MapPost("/benchmark/result-heavy", async (int? requests, int? concurrency) =>
{
    await tieredBenchmarkGate.WaitAsync();
    try
    {
        return Results.Ok(await TieredComparison.RunResultHeavyAsync(
            workerPath,
            Math.Clamp(requests ?? 50, 1, 10_000),
            Math.Clamp(concurrency ?? 4, 1, 8)));
    }
    finally
    {
        tieredBenchmarkGate.Release();
    }
});
app.MapPost("/benchmark/native-boundary-breakdown", async (int? requests) =>
{
    await tieredBenchmarkGate.WaitAsync();
    try
    {
        return Results.Ok(await NativeBoundaryBreakdown.RunAsync(
            stylesheet,
            Math.Clamp(requests ?? 250, 1, 10_000)));
    }
    finally
    {
        tieredBenchmarkGate.Release();
    }
});
app.MapPost("/benchmark/isolated-boundary-breakdown", async (int? requests) =>
{
    await tieredBenchmarkGate.WaitAsync();
    try
    {
        return Results.Ok(await IsolatedBoundaryBreakdown.RunAsync(
            workerPath,
            stylesheet,
            Math.Clamp(requests ?? 250, 1, 10_000)));
    }
    finally
    {
        tieredBenchmarkGate.Release();
    }
});
app.MapPost("/benchmark/isolated-batch", async (int? requests, int? orderOffset) =>
{
    await tieredBenchmarkGate.WaitAsync();
    try
    {
        return Results.Ok(await IsolatedBatchComparison.RunAsync(
            workerPath,
            stylesheet,
            Math.Clamp(requests ?? 250, 1, 10_000),
            orderOffset ?? 0));
    }
    finally
    {
        tieredBenchmarkGate.Release();
    }
});
app.MapPost("/benchmark/isolated-batch-worker-sweep", async (int? members, int? orderOffset) =>
{
    await tieredBenchmarkGate.WaitAsync();
    try
    {
        return Results.Ok(await IsolatedBatchWorkerSweep.RunAsync(
            workerPath,
            stylesheet,
            Math.Clamp(members ?? 1024, 128, 65_536),
            orderOffset ?? 0));
    }
    finally
    {
        tieredBenchmarkGate.Release();
    }
});
app.MapPost("/benchmark/isolated-batch-result-pressure", async (int? members, int? orderOffset) =>
{
    await tieredBenchmarkGate.WaitAsync();
    try
    {
        return Results.Ok(await IsolatedBatchResultPressureSweep.RunAsync(
            workerPath,
            Math.Clamp(members ?? 256, 128, 8_192),
            orderOffset ?? 0));
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
app.MapPost("/experiment/isolated-batch-controls", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        var outcomes = await worker.TransformControlledBatchAsync(
        [
            new("batch-control-before"),
            new("batch-control-cancelled", Cancelled: true),
            new("batch-control-limited", MaximumXsltInstructions: 0),
            new("batch-control-after")
        ]);
        var incremental = await worker.TransformControlledIncrementalBatchAsync(
        [
            new("batch-incremental-before"),
            new("batch-incremental-cancelled", Cancelled: true),
            new("batch-incremental-limited", MaximumXsltInstructions: 0),
            new("batch-incremental-after")
        ]);
        var recovery = await worker.TransformAsync("batch-control-recovery");
        return Results.Ok(new
        {
            outcomes = outcomes.Select(outcome => new
            {
                outcome.RequestIdentity,
                outcome.Result,
                failureCode = outcome.Failure?.Code,
                failureCategory = outcome.Failure?.Category,
                failureDetail = outcome.Failure?.Detail
            }),
            incrementalOutcomes = incremental.Outcomes.Select(outcome => new
            {
                outcome.RequestIdentity,
                outcome.Result,
                failureCode = outcome.Failure?.Code,
                failureCategory = outcome.Failure?.Category,
                failureDetail = outcome.Failure?.Detail
            }),
            incrementalFirstOutcomeMicroseconds = incremental.FirstOutcomeElapsed.TotalMicroseconds,
            incrementalFinalOutcomeMicroseconds = incremental.FinalOutcomeElapsed.TotalMicroseconds,
            recovery,
            memberControlsAreIndependent = true,
            incrementalMemberControlsAreIndependent = true,
            activeMidMemberCancellationIncluded = false
        });
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
app.MapPost("/experiment/isolated-incremental-batch-limit", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await IncrementalBatchOperationalExperiment
            .ExerciseCumulativeLimitAsync(workerPath));
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
app.MapPost("/experiment/isolated-incremental-batch-backpressure", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await IncrementalBatchOperationalExperiment
            .ExerciseSlowConsumerAsync(workerPath));
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
app.MapPost("/experiment/isolated-incremental-batch-consumer", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await IncrementalBatchOperationalExperiment
            .ExerciseConsumerBoundaryAsync(workerPath));
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
app.MapPost("/experiment/isolated-batch-loss", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await OperationalExperiments.ExerciseBatchLossClassificationAsync(
            workerPath,
            source,
            stylesheet));
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
app.MapPost("/experiment/isolated-batch-cancellation-races", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await OperationalExperiments.MeasureBatchNaturalCancellationRacesAsync(
            workerPath,
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
app.MapPost("/experiment/managed-cancellation", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await OperationalExperiments.ExerciseManagedCancellationAsync(
            workerPath,
            stylesheet));
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
app.MapPost("/experiment/diagnostic-parity", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await OperationalExperiments.ExerciseDiagnosticParityAsync(
            workerPath,
            source,
            stylesheet));
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
app.MapPost("/experiment/instruction-budget", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await OperationalExperiments.ExerciseInstructionBudgetAsync(
            workerPath,
            source,
            stylesheet));
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
app.MapPost("/experiment/native-boundary", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await OperationalExperiments.ExerciseNativeBoundaryAsync(
            source,
            stylesheet));
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
app.MapPost("/experiment/native-generation-replacement", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await OperationalExperiments.ExerciseNativeGenerationReplacementAsync(
            stylesheet));
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
app.MapPost("/experiment/native-registry-pressure", async (
    int? items,
    int? concurrency,
    int? generations,
    int? delayedOutcomes,
    int? settlementMilliseconds) =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await NativeRegistryPressureExperiment.RunAsync(
            stylesheet,
            items ?? 500,
            concurrency ?? 4,
            generations ?? 2,
            delayedOutcomes ?? 64,
            settlementMilliseconds ?? 1_000));
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
app.MapPost("/experiment/native-registry-bursts", async (
    int? concurrency,
    int? delayedFailures,
    int? largeOutcomes,
    int? largePayloadBytes) =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await NativeRegistryBurstExperiment.RunAsync(
            stylesheet,
            concurrency ?? 8,
            delayedFailures ?? 128,
            largeOutcomes ?? 8,
            largePayloadBytes ?? 900_000));
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
app.MapPost("/experiment/native-registry-replacement-soak", async (
    int? concurrency,
    int? replacements,
    int? retainedOldGenerations,
    int? requestsPerGeneration) =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await NativeRegistryReplacementSoak.RunAsync(
            stylesheet,
            concurrency ?? 8,
            replacements ?? 32,
            retainedOldGenerations ?? 2,
            requestsPerGeneration ?? 16));
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
app.MapPost("/experiment/native-active-cancellation", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await OperationalExperiments.ExerciseNativeActiveCancellationAsync(
            stylesheet));
    }
    finally
    {
        operationalExperimentGate.Release();
    }
});
app.MapPost("/experiment/native-natural-cancellation-races", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await OperationalExperiments.MeasureNativeNaturalCancellationRacesAsync(
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
app.MapPost("/experiment/worker-control-frame-serialization", async () =>
{
    await operationalExperimentGate.WaitAsync();
    try
    {
        return Results.Ok(await WorkerControlFrameExperiment.ExerciseAsync());
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
app.Lifetime.ApplicationStopping.Register(native.Dispose);
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
