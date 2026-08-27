public sealed class NativeFastXsltGenerationHost : IDisposable
{
    private readonly object _generationLock = new();
    private Generation? _current;
    private bool _disposed;

    private NativeFastXsltGenerationHost(Generation initial) => _current = initial;

    public static NativeFastXsltGenerationHost Create(
        string generationIdentity,
        string sourceIdentity,
        byte[] source,
        string stylesheetIdentity,
        byte[] stylesheet,
        int engines) => new(new Generation(
            generationIdentity,
            NativeFastXsltPool.Create(
                sourceIdentity,
                source,
                stylesheetIdentity,
                stylesheet,
                engines)));

    public async Task<GenerationTransformResult> TransformAsync(string requestIdentity)
    {
        using var lease = AcquireCurrent();
        return new GenerationTransformResult(
            lease.Identity,
            requestIdentity,
            await lease.Pool.TransformAsync(requestIdentity));
    }

    public string Replace(
        string generationIdentity,
        string sourceIdentity,
        byte[] source,
        string stylesheetIdentity,
        byte[] stylesheet,
        int engines)
    {
        var replacement = new Generation(
            generationIdentity,
            NativeFastXsltPool.Create(
                sourceIdentity,
                source,
                stylesheetIdentity,
                stylesheet,
                engines));
        Generation prior;
        lock (_generationLock)
        {
            if (_disposed)
            {
                replacement.Retire();
                throw new ObjectDisposedException(nameof(NativeFastXsltGenerationHost));
            }
            prior = _current ?? throw new InvalidOperationException("No active native generation.");
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
            return (_current ?? throw new InvalidOperationException("No active native generation."))
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

    private sealed class Generation(string identity, NativeFastXsltPool pool)
    {
        private readonly object _leaseLock = new();
        private int _leases;
        private bool _retired;
        private bool _disposed;

        public string Identity { get; } = identity;
        public NativeFastXsltPool Pool { get; } = pool;

        public GenerationLease Acquire()
        {
            lock (_leaseLock)
            {
                if (_retired)
                {
                    throw new InvalidOperationException("Cannot acquire a retired native generation.");
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
        NativeFastXsltPool pool,
        Action release) : IDisposable
    {
        private Action? _release = release;

        public string Identity { get; } = identity;
        public NativeFastXsltPool Pool { get; } = pool;

        public void Dispose() => Interlocked.Exchange(ref _release, null)?.Invoke();
    }
}
