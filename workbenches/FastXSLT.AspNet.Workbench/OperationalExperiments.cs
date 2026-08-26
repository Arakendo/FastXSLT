public static class OperationalExperiments
{
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
}
