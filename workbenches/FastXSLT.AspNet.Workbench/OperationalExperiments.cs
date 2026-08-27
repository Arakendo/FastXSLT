public static class OperationalExperiments
{
    private static readonly byte[] ReplacementSourceOne =
        "<order><order-item price=\"1.00\" qty=\"1\"/></order>"u8.ToArray();
    private static readonly byte[] ReplacementSourceTwo =
        "<order><order-item price=\"1.00\" qty=\"1\"/><order-item price=\"1.00\" qty=\"1\"/></order>"u8.ToArray();
    private static readonly byte[] UnsupportedMessageStylesheet =
        """<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template match="/"><xsl:message/></xsl:template></xsl:stylesheet>"""u8.ToArray();

    public static async Task<object> ExerciseWorkerRecoveryAsync(
        string workerPath,
        byte[] source,
        byte[] stylesheet)
    {
        using var pool = await FastXsltWorkerPool.StartAsync(
            workerPath,
            "urn:w3c:xslt30:for-004:source",
            source,
            "urn:w3c:xslt30:for-004:stylesheet",
            stylesheet,
            workers: 2);
        var sibling = pool.TransformAsync("recovery-sibling");
        var recovery = await pool.ExerciseTerminationAndRecoveryAsync(
            "recovery-failed",
            "recovery-after-replacement");
        return new
        {
            recovery,
            siblingRequestIdentity = "recovery-sibling",
            siblingResult = await sibling,
            failedRequestRetried = false,
            sealedGenerationReused = true
        };
    }

    public static async Task<object> ExerciseCooperativeCancellationAsync(
        string workerPath,
        byte[] source,
        byte[] stylesheet)
    {
        using var pool = await FastXsltWorkerPool.StartAsync(
            workerPath,
            "urn:w3c:xslt30:for-004:source",
            source,
            "urn:w3c:xslt30:for-004:stylesheet",
            stylesheet,
            workers: 1);
        var cancellation = await pool.ExercisePreDispatchCancellationAsync(
            "cooperative-cancelled",
            "cooperative-after-cancel");
        return new
        {
            cancellation,
            cancellationWasCooperative = true,
            workerWasTerminated = false,
            activeMidExecutionSignalSupported = false
        };
    }

    public static async Task<object> ExerciseActiveCancellationAsync(
        string workerPath,
        byte[] stylesheet)
    {
        var source = BuildCancellationSource(500);
        using var pool = await FastXsltWorkerPool.StartAsync(
            workerPath,
            "urn:fastxslt:active-cancellation:source",
            source,
            "urn:fastxslt:active-cancellation:stylesheet",
            stylesheet,
            workers: 1);
        var cancellation = await pool.ExerciseActiveCancellationAsync(
            "active-cooperative-cancelled",
            "active-cooperative-after-cancel");
        return new
        {
            cancellation,
            sourceItems = 500,
            signalSentAfterWorkerStarted = true,
            cancellationWasCooperative = true,
            workerWasTerminated = false,
            completionWinsIfCommittedBeforeSignal = true,
            firstChargeBarrierWasExperimental = true
        };
    }

    public static async Task<object> MeasureNaturalCancellationRacesAsync(
        string workerPath,
        byte[] stylesheet)
    {
        var source = BuildCancellationSource(20_000);
        const string expected = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>20000.00</out>";
        using var pool = await FastXsltWorkerPool.StartAsync(
            workerPath,
            "urn:fastxslt:natural-cancellation:source",
            source,
            "urn:fastxslt:natural-cancellation:stylesheet",
            stylesheet,
            workers: 1);
        var races = await pool.MeasureUnpausedCancellationRacesAsync(
            "natural-cancellation",
            expected,
            "natural-cancellation-recovery",
            trials: 25);
        return new
        {
            races,
            sourceItems = 20_000,
            firstChargeBarrierUsed = false,
            completionWinsIfCommittedBeforeSignal = true,
            workerWasTerminated = false
        };
    }

    public static async Task<object> ExerciseManagedCancellationAsync(
        string workerPath,
        byte[] stylesheet)
    {
        var source = BuildCancellationSource(20_000);
        using var pool = await FastXsltWorkerPool.StartAsync(
            workerPath,
            "urn:fastxslt:managed-cancellation:source",
            source,
            "urn:fastxslt:managed-cancellation:stylesheet",
            stylesheet,
            workers: 1);
        var cancellation = await pool.ExerciseManagedCancellationAsync(
            "managed-cancellation-pre-dispatch",
            "managed-cancellation-active",
            "managed-cancellation-recovery");
        return new
        {
            cancellation,
            sourceItems = 20_000,
            activeOutcome = cancellation.ActiveFailureCode is null ? "completed" : "cancelled",
            completionWinsIfCommittedBeforeSignal = true,
            managedTokenMeansCooperativeRequest = true,
            hardTerminationGuaranteed = false
        };
    }

    public static async Task<object> ExerciseDiagnosticParityAsync(
        string workerPath,
        byte[] source,
        byte[] stylesheet)
    {
        using var client = await FastXsltWorkerClient.StartAsync(
            workerPath,
            "urn:fastxslt:diagnostic:source",
            source,
            "urn:fastxslt:diagnostic:stylesheet",
            stylesheet);
        var processId = client.ProcessId;
        var invalidIdentity = await CaptureFailureAsync(() => client.TransformAsync(""));
        using var cancelled = new CancellationTokenSource();
        cancelled.Cancel();
        var cancellation = await CaptureFailureAsync(
            () => client.TransformAsync("diagnostic-cancelled", cancelled.Token));
        var recoveryResult = await client.TransformAsync("diagnostic-recovery");

        var malformedSource = await CaptureInitializationFailureAsync(
            workerPath,
            "urn:fastxslt:diagnostic:malformed-source",
            "<order></other>"u8.ToArray(),
            "urn:fastxslt:diagnostic:stylesheet",
            stylesheet);
        var unsupportedStylesheet = await CaptureInitializationFailureAsync(
            workerPath,
            "urn:fastxslt:diagnostic:source",
            source,
            "urn:fastxslt:diagnostic:unsupported-stylesheet",
            UnsupportedMessageStylesheet);

        return new
        {
            invalidIdentity,
            malformedSource,
            unsupportedStylesheet,
            cancellation,
            processIdBefore = processId,
            processIdAfter = client.ProcessId,
            recoveryResult,
            sameDiagnosticFieldsAsDirectRustAssertions = true
        };
    }

    public static async Task<object> ExerciseInstructionBudgetAsync(
        string workerPath,
        byte[] source,
        byte[] stylesheet)
    {
        using var client = await FastXsltWorkerClient.StartAsync(
            workerPath,
            "urn:w3c:xslt30:for-004:source",
            source,
            "urn:w3c:xslt30:for-004:stylesheet",
            stylesheet);
        var processId = client.ProcessId;
        var exhaustion = await CaptureFailureAsync(
            () => client.TransformWithXsltInstructionLimitAsync(
                "instruction-budget-exhausted",
                maximumXsltInstructions: 0));
        var recoveryResult = await client.TransformAsync("instruction-budget-recovery");
        return new
        {
            exhaustion,
            configuredMaximumXsltInstructions = 0,
            processIdBefore = processId,
            processIdAfter = client.ProcessId,
            recoveryResult,
            deterministicEngineBudget = true,
            cooperativeCancellation = false,
            workerWasTerminated = false,
            requestWasRetried = false
        };
    }

    public static async Task<object> ExerciseNativeBoundaryAsync(
        byte[] source,
        byte[] stylesheet)
    {
        using var client = NativeFastXsltClient.Create(
            "urn:fastxslt:native-boundary:source",
            source,
            "urn:fastxslt:native-boundary:stylesheet",
            stylesheet);
        NativeFastXsltException invalidIdentity;
        try
        {
            _ = client.Transform("");
            throw new InvalidOperationException("Empty native request identity unexpectedly succeeded.");
        }
        catch (NativeFastXsltException failure)
        {
            invalidIdentity = failure;
        }
        var recoveryResult = client.Transform("native-boundary-recovery");

        NativeFastXsltException cancellation;
        try
        {
            _ = client.TransformWithInvocationPolicy(
                "native-controlled-cancelled",
                cancellationRequested: true,
                maximumXsltInstructions: 1_000_000);
            throw new InvalidOperationException("Pre-signalled native cancellation unexpectedly succeeded.");
        }
        catch (NativeFastXsltException failure)
        {
            cancellation = failure;
        }

        NativeFastXsltException instructionBudget;
        try
        {
            _ = client.TransformWithInvocationPolicy(
                "native-instruction-budget",
                cancellationRequested: false,
                maximumXsltInstructions: 0);
            throw new InvalidOperationException("Zero native instruction budget unexpectedly succeeded.");
        }
        catch (NativeFastXsltException failure)
        {
            instructionBudget = failure;
        }
        var controlledRecoveryResult = client.Transform("native-controlled-recovery");

        NativeFastXsltException malformedSource;
        try
        {
            using var unexpected = NativeFastXsltClient.Create(
                "urn:fastxslt:native-boundary:malformed-source",
                "<order></other>"u8.ToArray(),
                "urn:fastxslt:native-boundary:stylesheet",
                stylesheet);
            throw new InvalidOperationException("Malformed native source unexpectedly initialized.");
        }
        catch (NativeFastXsltException failure)
        {
            malformedSource = failure;
        }

        NativeFastXsltException unsupportedStylesheet;
        try
        {
            using var unexpected = NativeFastXsltClient.Create(
                "urn:fastxslt:native-boundary:unsupported-source",
                source,
                "urn:fastxslt:diagnostic:unsupported-stylesheet",
                UnsupportedMessageStylesheet);
            throw new InvalidOperationException("Unsupported native stylesheet unexpectedly initialized.");
        }
        catch (NativeFastXsltException failure)
        {
            unsupportedStylesheet = failure;
        }

        using var first = NativeFastXsltClient.Create(
            "urn:fastxslt:native-boundary:concurrent-source-1",
            source,
            "urn:fastxslt:native-boundary:concurrent-stylesheet-1",
            stylesheet);
        using var second = NativeFastXsltClient.Create(
            "urn:fastxslt:native-boundary:concurrent-source-2",
            source,
            "urn:fastxslt:native-boundary:concurrent-stylesheet-2",
            stylesheet);
        var concurrentResults = await Task.WhenAll(
            Task.Run(() => first.Transform("native-concurrent-1")),
            Task.Run(() => second.Transform("native-concurrent-2")));

        var disposed = NativeFastXsltClient.Create(
            "urn:fastxslt:native-boundary:dispose-source",
            source,
            "urn:fastxslt:native-boundary:dispose-stylesheet",
            stylesheet);
        disposed.Dispose();
        disposed.Dispose();
        var useAfterDisposeRejected = false;
        try
        {
            _ = disposed.Transform("native-after-dispose");
        }
        catch (ObjectDisposedException)
        {
            useAfterDisposeRejected = true;
        }

        return new
        {
            invalidIdentity = new DiagnosticEvidence(
                invalidIdentity.Code,
                invalidIdentity.Category,
                invalidIdentity.RequestId,
                invalidIdentity.Detail),
            malformedSource = new DiagnosticEvidence(
                malformedSource.Code,
                malformedSource.Category,
                malformedSource.RequestId,
                malformedSource.Detail),
            unsupportedStylesheet = new DiagnosticEvidence(
                unsupportedStylesheet.Code,
                unsupportedStylesheet.Category,
                unsupportedStylesheet.RequestId,
                unsupportedStylesheet.Detail),
            cancellation = new DiagnosticEvidence(
                cancellation.Code,
                cancellation.Category,
                cancellation.RequestId,
                cancellation.Detail),
            instructionBudget = new DiagnosticEvidence(
                instructionBudget.Code,
                instructionBudget.Category,
                instructionBudget.RequestId,
                instructionBudget.Detail),
            recoveryResult,
            controlledRecoveryResult,
            concurrentResults,
            independentHandlesExecutedConcurrently = true,
            controlsWereScalarAndPreDispatch = true,
            activeMidExecutionSignalSupported = false,
            hardTerminationGuaranteed = false,
            doubleDisposeWasIdempotent = true,
            useAfterDisposeRejected
        };
    }

    public static async Task<object> ExerciseNativeGenerationReplacementAsync(
        byte[] stylesheet)
    {
        using var host = NativeFastXsltGenerationHost.Create(
            "native-generation-001",
            "urn:fastxslt:native-generation:source:g1",
            ReplacementSourceOne,
            "urn:fastxslt:native-generation:stylesheet:g1",
            stylesheet,
            engines: 1);
        using var oldLease = host.AcquireCurrent();
        var oldPool = oldLease.Pool;
        var retiredIdentity = host.Replace(
            "native-generation-002",
            "urn:fastxslt:native-generation:source:g2",
            ReplacementSourceTwo,
            "urn:fastxslt:native-generation:stylesheet:g2",
            stylesheet,
            engines: 1);
        var newGeneration = await host.TransformAsync("native-replacement-new");
        var oldResult = await oldPool.TransformAsync("native-replacement-old-in-flight");
        oldLease.Dispose();
        var oldGenerationDisposedAfterLeaseRelease = false;
        try
        {
            _ = await oldPool.TransformAsync("native-replacement-after-drain");
        }
        catch (ObjectDisposedException)
        {
            oldGenerationDisposedAfterLeaseRelease = true;
        }
        return new
        {
            retiredGenerationIdentity = retiredIdentity,
            oldLeaseGenerationIdentity = oldLease.Identity,
            oldRequestIdentity = "native-replacement-old-in-flight",
            oldResult,
            newGeneration,
            replacementInitializedBeforePromotion = true,
            promotionWasExplicit = true,
            oldGenerationDisposedAfterLeaseRelease
        };
    }

    private static async Task<DiagnosticEvidence> CaptureInitializationFailureAsync(
        string workerPath,
        string sourceIdentity,
        byte[] source,
        string stylesheetIdentity,
        byte[] stylesheet) => await CaptureFailureAsync(async () =>
        {
            using var unexpected = await FastXsltWorkerClient.StartAsync(
                workerPath,
                sourceIdentity,
                source,
                stylesheetIdentity,
                stylesheet);
            return "unexpected initialization";
        });

    private static async Task<DiagnosticEvidence> CaptureFailureAsync(
        Func<Task<string>> operation)
    {
        try
        {
            _ = await operation();
            throw new InvalidOperationException("Diagnostic probe unexpectedly succeeded.");
        }
        catch (FastXsltWorkerException failure)
        {
            return new DiagnosticEvidence(
                failure.Code,
                failure.Category,
                failure.RequestId,
                failure.Detail);
        }
    }

    public static async Task<object> ExerciseGenerationReplacementAsync(
        string workerPath,
        byte[] source,
        byte[] stylesheet)
    {
        using var host = await FastXsltWorkerGenerationHost.StartAsync(
            "generation-001",
            workerPath,
            "urn:w3c:xslt30:for-004:source:g1",
            source,
            "urn:w3c:xslt30:for-004:stylesheet:g1",
            stylesheet,
            workers: 1);
        using var oldLease = host.AcquireCurrent();
        var retiredIdentity = await host.ReplaceAsync(
            "generation-002",
            workerPath,
            "urn:w3c:xslt30:for-004:source:g2",
            source,
            "urn:w3c:xslt30:for-004:stylesheet:g2",
            stylesheet,
            workers: 1);
        var newGeneration = await host.TransformAsync("replacement-new");
        var oldResult = await oldLease.Pool.TransformAsync("replacement-old-in-flight");
        return new
        {
            retiredGenerationIdentity = retiredIdentity,
            oldLeaseGenerationIdentity = oldLease.Identity,
            oldRequestIdentity = "replacement-old-in-flight",
            oldResult,
            newGeneration,
            promotionWasExplicit = true,
            oldGenerationDrainsOnLeaseRelease = true
        };
    }

    public static async Task<object> ExerciseHostFileReplacementAsync(
        string workerPath,
        string scratchRoot,
        byte[] stylesheet)
    {
        var experimentDirectory = Path.Combine(
            scratchRoot,
            $"resource-replacement-{Guid.NewGuid():N}");
        Directory.CreateDirectory(experimentDirectory);
        var sourcePath = Path.Combine(experimentDirectory, "source.xml");
        var stylesheetPath = Path.Combine(experimentDirectory, "stylesheet.xsl");
        var retiredSourcePath = Path.Combine(experimentDirectory, "source.retired.xml");
        var retiredStylesheetPath = Path.Combine(experimentDirectory, "stylesheet.retired.xsl");
        try
        {
            await File.WriteAllBytesAsync(sourcePath, ReplacementSourceOne);
            await File.WriteAllBytesAsync(stylesheetPath, stylesheet);
            var firstSource = await ImportAndCloseAsync(sourcePath);
            var firstStylesheet = await ImportAndCloseAsync(stylesheetPath);
            using var host = await FastXsltWorkerGenerationHost.StartAsync(
                "file-generation-001",
                workerPath,
                "urn:fastxslt:file-replacement:source:g1",
                firstSource,
                "urn:fastxslt:file-replacement:stylesheet:g1",
                firstStylesheet,
                workers: 1);
            using var oldLease = host.AcquireCurrent();

            File.Move(sourcePath, retiredSourcePath);
            File.Move(stylesheetPath, retiredStylesheetPath);
            await File.WriteAllBytesAsync(sourcePath, ReplacementSourceTwo);
            await File.WriteAllBytesAsync(stylesheetPath, stylesheet);
            File.Delete(retiredSourcePath);
            File.Delete(retiredStylesheetPath);

            var secondSource = await ImportAndCloseAsync(sourcePath);
            var secondStylesheet = await ImportAndCloseAsync(stylesheetPath);
            var retiredGeneration = await host.ReplaceAsync(
                "file-generation-002",
                workerPath,
                "urn:fastxslt:file-replacement:source:g2",
                secondSource,
                "urn:fastxslt:file-replacement:stylesheet:g2",
                secondStylesheet,
                workers: 1);
            var newGeneration = await host.TransformAsync("file-replacement-new");
            var oldResult = await oldLease.Pool.TransformAsync("file-replacement-old-in-flight");
            oldLease.Dispose();
            return new
            {
                retiredGeneration,
                oldGenerationIdentity = oldLease.Identity,
                oldResult,
                newGeneration,
                importedHandlesClosedBeforePromotion = true,
                originalFilesRenamedAndRemovedWhileGenerationWasLive = true,
                sourceBytesChanged = !firstSource.AsSpan().SequenceEqual(secondSource)
            };
        }
        finally
        {
            Directory.Delete(experimentDirectory, recursive: true);
        }
    }

    private static async Task<byte[]> ImportAndCloseAsync(string path)
    {
        await using var input = new FileStream(
            path,
            FileMode.Open,
            FileAccess.Read,
            FileShare.Read,
            bufferSize: 4_096,
            useAsync: true);
        using var imported = new MemoryStream();
        await input.CopyToAsync(imported);
        return imported.ToArray();
    }

    private static byte[] BuildCancellationSource(int items)
    {
        var source = new System.Text.StringBuilder("<order>");
        for (var index = 0; index < items; index++)
        {
            source.Append("<order-item price=\"1.00\" qty=\"1\"/>");
        }
        source.Append("</order>");
        return System.Text.Encoding.UTF8.GetBytes(source.ToString());
    }
}

public sealed record DiagnosticEvidence(
    string Code,
    string Category,
    string? RequestId,
    string Detail);
