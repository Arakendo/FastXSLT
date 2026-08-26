public sealed class FastXsltWorkerPool : IDisposable
{
    private readonly WorkerBootstrap _bootstrap;
    private readonly WorkerSlot[] _workers;
    private readonly Queue<WorkerSlot> _available;
    private readonly SemaphoreSlim _slots;
    private readonly object _queueLock = new();
    private bool _disposed;

    private FastXsltWorkerPool(WorkerBootstrap bootstrap, WorkerSlot[] workers)
    {
        _bootstrap = bootstrap;
        _workers = workers;
        _available = new Queue<WorkerSlot>(workers);
        _slots = new SemaphoreSlim(workers.Length, workers.Length);
    }

    public static async Task<FastXsltWorkerPool> StartAsync(
        string workerPath,
        string sourceIdentity,
        byte[] source,
        string stylesheetIdentity,
        byte[] stylesheet,
        int workers)
    {
        ArgumentOutOfRangeException.ThrowIfLessThan(workers, 1);
        var bootstrap = new WorkerBootstrap(
            workerPath,
            sourceIdentity,
            (byte[])source.Clone(),
            stylesheetIdentity,
            (byte[])stylesheet.Clone());
        var started = new List<WorkerSlot>(workers);
        try
        {
            for (var index = 0; index < workers; index++)
            {
                started.Add(new WorkerSlot(await bootstrap.StartAsync()));
            }
            return new FastXsltWorkerPool(bootstrap, started.ToArray());
        }
        catch
        {
            foreach (var worker in started)
            {
                worker.Client.Dispose();
            }
            throw;
        }
    }

    public async Task<string> TransformAsync(string requestIdentity)
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        await _slots.WaitAsync();
        WorkerSlot worker;
        lock (_queueLock)
        {
            worker = _available.Dequeue();
        }
        try
        {
            try
            {
                return await worker.Client.TransformAsync(requestIdentity);
            }
            catch (FastXsltWorkerException)
            {
                throw;
            }
            catch (Exception failure) when (IsWorkerBoundaryFailure(failure))
            {
                var formerProcessId = worker.Client.ProcessId;
                await ReplaceAsync(worker);
                throw new FastXsltWorkerOperationalException(
                    "FXWB2001",
                    "worker-terminated",
                    requestIdentity,
                    formerProcessId,
                    worker.Client.ProcessId,
                    "The isolated worker ended during the request. The request was not retried; " +
                    "the slot was initialized from the same sealed generation.",
                    failure);
            }
        }
        finally
        {
            lock (_queueLock)
            {
                _available.Enqueue(worker);
            }
            _slots.Release();
        }
    }

    public async Task<WorkerRecoveryEvidence> ExerciseTerminationAndRecoveryAsync(
        string failedRequestIdentity,
        string recoveryRequestIdentity)
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        await _slots.WaitAsync();
        WorkerSlot worker;
        lock (_queueLock)
        {
            worker = _available.Dequeue();
        }
        try
        {
            var formerProcessId = worker.Client.ProcessId;
            await worker.Client.BeginNonCooperatingProbeAsync(failedRequestIdentity);
            worker.Client.TerminateForExperiment();
            await ReplaceAsync(worker);
            var disposition = new FastXsltWorkerOperationalException(
                "FXWB2001",
                "worker-terminated",
                failedRequestIdentity,
                formerProcessId,
                worker.Client.ProcessId,
                "The acknowledged non-cooperating request was terminated with its worker and was not retried.",
                new IOException("The workbench supervisor deliberately terminated the isolated worker."));
            var result = await worker.Client.TransformAsync(recoveryRequestIdentity);
            return new WorkerRecoveryEvidence(
                disposition.Code,
                disposition.Category,
                disposition.RequestId,
                formerProcessId,
                worker.Client.ProcessId,
                recoveryRequestIdentity,
                result);
        }
        finally
        {
            lock (_queueLock)
            {
                _available.Enqueue(worker);
            }
            _slots.Release();
        }
    }

    public async Task<CooperativeCancellationEvidence> ExercisePreDispatchCancellationAsync(
        string cancelledRequestIdentity,
        string recoveryRequestIdentity)
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        await _slots.WaitAsync();
        WorkerSlot worker;
        lock (_queueLock)
        {
            worker = _available.Dequeue();
        }
        try
        {
            var processId = worker.Client.ProcessId;
            FastXsltWorkerException disposition;
            try
            {
                await worker.Client.TransformCancelledBeforeDispatchAsync(cancelledRequestIdentity);
                throw new InvalidOperationException("A signalled invocation unexpectedly completed.");
            }
            catch (FastXsltWorkerException failure)
            {
                disposition = failure;
            }
            var recoveryResult = await worker.Client.TransformAsync(recoveryRequestIdentity);
            return new CooperativeCancellationEvidence(
                disposition.Code,
                disposition.Category,
                disposition.RequestId,
                disposition.Detail,
                processId,
                worker.Client.ProcessId,
                recoveryRequestIdentity,
                recoveryResult);
        }
        finally
        {
            lock (_queueLock)
            {
                _available.Enqueue(worker);
            }
            _slots.Release();
        }
    }

    public async Task<ActiveCancellationEvidence> ExerciseActiveCancellationAsync(
        string cancelledRequestIdentity,
        string recoveryRequestIdentity)
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        await _slots.WaitAsync();
        WorkerSlot worker;
        lock (_queueLock)
        {
            worker = _available.Dequeue();
        }
        try
        {
            var processId = worker.Client.ProcessId;
            var invocation = await worker.Client.StartControlledTransformAsync(cancelledRequestIdentity);
            await invocation.SendUnrelatedCancellationForExperimentAsync("unrelated-cancellation");
            await Task.Delay(10);
            var unrelatedSignalIgnored = !invocation.Completion.IsCompleted;
            var observation = System.Diagnostics.Stopwatch.StartNew();
            await invocation.CancelAsync();
            FastXsltWorkerException disposition;
            try
            {
                _ = await invocation.Completion;
                throw new InvalidOperationException("Cancellation lost to completion in the active probe.");
            }
            catch (FastXsltWorkerException failure)
            {
                disposition = failure;
            }
            observation.Stop();
            var recoveryResult = await worker.Client.TransformAsync(recoveryRequestIdentity);
            return new ActiveCancellationEvidence(
                disposition.Code,
                disposition.Category,
                disposition.RequestId,
                disposition.Detail,
                observation.Elapsed.TotalMilliseconds,
                unrelatedSignalIgnored,
                processId,
                worker.Client.ProcessId,
                recoveryRequestIdentity,
                recoveryResult);
        }
        finally
        {
            lock (_queueLock)
            {
                _available.Enqueue(worker);
            }
            _slots.Release();
        }
    }

    public (TimeSpan ProcessorTime, long WorkingSetBytes) ObserveProcesses()
    {
        var processorTime = TimeSpan.Zero;
        long workingSetBytes = 0;
        foreach (var worker in _workers)
        {
            var observation = worker.Client.ObserveProcess();
            processorTime += observation.ProcessorTime;
            workingSetBytes = checked(workingSetBytes + observation.WorkingSetBytes);
        }
        return (processorTime, workingSetBytes);
    }

    public void Dispose()
    {
        if (_disposed)
        {
            return;
        }
        _disposed = true;
        foreach (var worker in _workers)
        {
            worker.Client.Dispose();
        }
        _slots.Dispose();
    }


    private async Task ReplaceAsync(WorkerSlot worker)
    {
        var replacement = await _bootstrap.StartAsync();
        var prior = worker.Client;
        worker.Client = replacement;
        prior.Dispose();
    }

    private static bool IsWorkerBoundaryFailure(Exception failure) =>
        failure is IOException or InvalidOperationException or ObjectDisposedException;

    private sealed record WorkerBootstrap(
        string WorkerPath,
        string SourceIdentity,
        byte[] Source,
        string StylesheetIdentity,
        byte[] Stylesheet)
    {
        public Task<FastXsltWorkerClient> StartAsync() => FastXsltWorkerClient.StartAsync(
            WorkerPath,
            SourceIdentity,
            Source,
            StylesheetIdentity,
            Stylesheet);
    }

    private sealed class WorkerSlot(FastXsltWorkerClient client)
    {
        public FastXsltWorkerClient Client { get; set; } = client;
    }
}

public sealed record WorkerRecoveryEvidence(
    string FailureCode,
    string FailureCategory,
    string FailedRequestIdentity,
    int FormerProcessId,
    int ReplacementProcessId,
    string RecoveryRequestIdentity,
    string RecoveryResult);

public sealed record CooperativeCancellationEvidence(
    string FailureCode,
    string FailureCategory,
    string? CancelledRequestIdentity,
    string FailureDetail,
    int ProcessIdBefore,
    int ProcessIdAfter,
    string RecoveryRequestIdentity,
    string RecoveryResult);

public sealed record ActiveCancellationEvidence(
    string FailureCode,
    string FailureCategory,
    string? CancelledRequestIdentity,
    string FailureDetail,
    double SignalToObservationMilliseconds,
    bool UnrelatedSignalIgnored,
    int ProcessIdBefore,
    int ProcessIdAfter,
    string RecoveryRequestIdentity,
    string RecoveryResult);

public sealed class FastXsltWorkerOperationalException(
    string code,
    string category,
    string requestId,
    int formerProcessId,
    int replacementProcessId,
    string detail,
    Exception innerException) : Exception(detail, innerException)
{
    public string Code { get; } = code;
    public string Category { get; } = category;
    public string RequestId { get; } = requestId;
    public int FormerProcessId { get; } = formerProcessId;
    public int ReplacementProcessId { get; } = replacementProcessId;
}
