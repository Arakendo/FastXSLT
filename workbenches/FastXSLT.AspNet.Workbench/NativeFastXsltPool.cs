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
