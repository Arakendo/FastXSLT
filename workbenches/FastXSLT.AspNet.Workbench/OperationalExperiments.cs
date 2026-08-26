public static class OperationalExperiments
{
    private static readonly byte[] ReplacementSourceOne =
        "<order><order-item price=\"1.00\" qty=\"1\"/></order>"u8.ToArray();
    private static readonly byte[] ReplacementSourceTwo =
        "<order><order-item price=\"1.00\" qty=\"1\"/><order-item price=\"1.00\" qty=\"1\"/></order>"u8.ToArray();

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
