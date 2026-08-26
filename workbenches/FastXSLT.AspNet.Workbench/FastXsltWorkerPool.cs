public sealed class FastXsltWorkerPool : IDisposable
{
    private readonly FastXsltWorkerClient[] _workers;
    private readonly Queue<FastXsltWorkerClient> _available;
    private readonly SemaphoreSlim _slots;
    private readonly object _queueLock = new();
    private bool _disposed;

    private FastXsltWorkerPool(FastXsltWorkerClient[] workers)
    {
        _workers = workers;
        _available = new Queue<FastXsltWorkerClient>(workers);
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
        var started = new List<FastXsltWorkerClient>(workers);
        try
        {
            for (var index = 0; index < workers; index++)
            {
                started.Add(await FastXsltWorkerClient.StartAsync(
                    workerPath,
                    sourceIdentity,
                    source,
                    stylesheetIdentity,
                    stylesheet));
            }
            return new FastXsltWorkerPool(started.ToArray());
        }
        catch
        {
            foreach (var worker in started)
            {
                worker.Dispose();
            }
            throw;
        }
    }

    public async Task<string> TransformAsync(string requestIdentity)
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        await _slots.WaitAsync();
        FastXsltWorkerClient worker;
        lock (_queueLock)
        {
            worker = _available.Dequeue();
        }
        try
        {
            return await worker.TransformAsync(requestIdentity);
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
            var observation = worker.ObserveProcess();
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
            worker.Dispose();
        }
        _slots.Dispose();
    }
}
