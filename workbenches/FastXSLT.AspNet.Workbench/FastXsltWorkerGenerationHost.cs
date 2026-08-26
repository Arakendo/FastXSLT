public sealed class FastXsltWorkerGenerationHost : IDisposable
{
    private readonly object _generationLock = new();
    private Generation? _current;
    private bool _disposed;

    private FastXsltWorkerGenerationHost(Generation initial)
    {
        _current = initial;
    }

    public static async Task<FastXsltWorkerGenerationHost> StartAsync(
        string generationIdentity,
        string workerPath,
        string sourceIdentity,
        byte[] source,
        string stylesheetIdentity,
        byte[] stylesheet,
        int workers) => new(new Generation(
            generationIdentity,
            await FastXsltWorkerPool.StartAsync(
                workerPath,
                sourceIdentity,
                source,
                stylesheetIdentity,
                stylesheet,
                workers)));

    public async Task<GenerationTransformResult> TransformAsync(string requestIdentity)
    {
        using var lease = AcquireCurrent();
        return new GenerationTransformResult(
            lease.Identity,
            requestIdentity,
            await lease.Pool.TransformAsync(requestIdentity));
    }

    public async Task<string> ReplaceAsync(
        string generationIdentity,
        string workerPath,
        string sourceIdentity,
        byte[] source,
        string stylesheetIdentity,
        byte[] stylesheet,
        int workers)
    {
        var replacement = new Generation(
            generationIdentity,
            await FastXsltWorkerPool.StartAsync(
                workerPath,
                sourceIdentity,
                source,
                stylesheetIdentity,
                stylesheet,
                workers));
        Generation prior;
        lock (_generationLock)
        {
            if (_disposed)
            {
                replacement.Retire();
                throw new ObjectDisposedException(nameof(FastXsltWorkerGenerationHost));
            }
            prior = _current ?? throw new InvalidOperationException("No active worker generation.");
            _current = replacement;
        }
        prior.Retire();
        return prior.Identity;
    }

    public GenerationLease AcquireCurrent()
    {
        lock (_generationLock)
        {
            ObjectDisposedException.ThrowIf(_disposed, this);
            return (_current ?? throw new InvalidOperationException("No active worker generation."))
                .Acquire();
        }
    }

    public void Dispose()
    {
        Generation? current;
        lock (_generationLock)
        {
            if (_disposed)
            {
                return;
            }
            _disposed = true;
            current = _current;
            _current = null;
        }
        current?.Retire();
    }

    private sealed class Generation(string identity, FastXsltWorkerPool pool)
    {
        private readonly object _leaseLock = new();
        private int _leases;
        private bool _retired;
        private bool _disposed;

        public string Identity { get; } = identity;
        public FastXsltWorkerPool Pool { get; } = pool;

        public GenerationLease Acquire()
        {
            lock (_leaseLock)
            {
                if (_retired)
                {
                    throw new InvalidOperationException("Cannot acquire a retired generation.");
                }
                _leases++;
                return new GenerationLease(Identity, Pool, Release);
            }
        }

        public void Retire()
        {
            lock (_leaseLock)
            {
                _retired = true;
                DisposeIfDrained();
            }
        }

        private void Release()
        {
            lock (_leaseLock)
            {
                _leases--;
                DisposeIfDrained();
            }
        }

        private void DisposeIfDrained()
        {
            if (_retired && _leases == 0 && !_disposed)
            {
                _disposed = true;
                Pool.Dispose();
            }
        }
    }

    public sealed class GenerationLease(
        string identity,
        FastXsltWorkerPool pool,
        Action release) : IDisposable
    {
        private Action? _release = release;

        public string Identity { get; } = identity;
        public FastXsltWorkerPool Pool { get; } = pool;

        public void Dispose() => Interlocked.Exchange(ref _release, null)?.Invoke();
    }
}

public sealed record GenerationTransformResult(
    string GenerationIdentity,
    string RequestIdentity,
    string Result);
