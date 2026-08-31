public sealed class NativeFastXsltPool : IDisposable
{
    private readonly NativeFastXsltClient[] _clients;
    private readonly Queue<NativeFastXsltClient> _available;
    private readonly SemaphoreSlim _slots;
    private readonly object _queueLock = new();
    private bool _disposed;

    private NativeFastXsltPool(NativeFastXsltClient[] clients)
    {
        _clients = clients;
        _available = new Queue<NativeFastXsltClient>(clients);
        _slots = new SemaphoreSlim(clients.Length, clients.Length);
    }

    public static NativeFastXsltPool Create(
        string sourceIdentity,
        byte[] source,
        string stylesheetIdentity,
        byte[] stylesheet,
        int engines)
    {
        ArgumentOutOfRangeException.ThrowIfLessThan(engines, 1);
        var clients = new List<NativeFastXsltClient>(engines);
        try
        {
            for (var index = 0; index < engines; index++)
            {
                clients.Add(NativeFastXsltClient.Create(
                    $"{sourceIdentity}:engine-{index}",
                    source,
                    $"{stylesheetIdentity}:engine-{index}",
                    stylesheet));
            }
            return new NativeFastXsltPool(clients.ToArray());
        }
        catch
        {
            foreach (var client in clients)
            {
                client.Dispose();
            }
            throw;
        }
    }

    public async Task<string> TransformAsync(string requestIdentity)
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        await _slots.WaitAsync();
        NativeFastXsltClient client;
        lock (_queueLock)
        {
            client = _available.Dequeue();
        }
        try
        {
            return client.Transform(requestIdentity);
        }
        finally
        {
            lock (_queueLock)
            {
                _available.Enqueue(client);
            }
            _slots.Release();
        }
    }

    public async Task<NativeFastXsltClient.NativeRetainedOutcome> TransformRetainedAsync(
        string requestIdentity)
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        await _slots.WaitAsync();
        NativeFastXsltClient client;
        lock (_queueLock)
        {
            client = _available.Dequeue();
        }
        try
        {
            return client.TransformRetained(requestIdentity);
        }
        finally
        {
            lock (_queueLock)
            {
                _available.Enqueue(client);
            }
            _slots.Release();
        }
    }

    public async Task<IReadOnlyList<NativeFastXsltException>> ExerciseActiveControlBurstAsync(
        string requestPrefix,
        TimeSpan observationTimeout,
        Action observeHighWater)
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        var transforms = _clients
            .Select((client, index) => client.StartBarrierTransform($"{requestPrefix}-{index}"))
            .ToArray();
        try
        {
            var waiting = System.Diagnostics.Stopwatch.StartNew();
            while (transforms.Any(transform => !transform.FirstChargeObserved) &&
                   transforms.All(transform => !transform.IsCompleted) &&
                   waiting.Elapsed < observationTimeout)
            {
                await Task.Delay(1);
            }
            if (transforms.Any(transform => !transform.FirstChargeObserved))
            {
                throw new TimeoutException(
                    "Native control burst did not reach every first-charge barrier.");
            }

            observeHighWater();
            foreach (var transform in transforms)
            {
                transform.Cancel();
            }
            var failures = await Task.WhenAll(
                transforms.Select(transform => transform.ObserveCancellationAsync()));
            if (failures.Any(failure =>
                failure.Code != "FXCT0001" || failure.Category != "cancelled"))
            {
                throw new InvalidOperationException(
                    "Native control burst returned a non-cancellation failure.");
            }
            return failures;
        }
        finally
        {
            foreach (var transform in transforms)
            {
                if (!transform.IsCompleted)
                {
                    transform.Cancel();
                }
            }
            await Task.WhenAll(
                transforms.Select(transform => transform.AwaitCompletionIgnoringFailureAsync()));
            foreach (var transform in transforms)
            {
                transform.Dispose();
            }
        }
    }

    public void Dispose()
    {
        if (_disposed)
        {
            return;
        }
        _disposed = true;
        foreach (var client in _clients)
        {
            client.Dispose();
        }
        _slots.Dispose();
    }
}
