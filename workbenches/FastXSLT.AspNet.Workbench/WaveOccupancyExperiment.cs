using System.Diagnostics;

public static class WaveOccupancyExperiment
{
    private static readonly int[] WaveSizes = [5, 8];

    public static async Task<WaveOccupancyReport> RunAsync(
        string workerPath,
        byte[] source,
        byte[] stylesheet,
        int waves,
        int rounds)
    {
        waves = Math.Clamp(waves, 100, 20_000);
        rounds = Math.Clamp(rounds, 1, 9);
        const int workers = 8;
        const string expected = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>36.02</out>";
        var clients = new List<FastXsltWorkerClient>(workers);
        var nativeClients = new List<NativeFastXsltClient>(workers);
        try
        {
            for (var index = 0; index < workers; index++)
            {
                clients.Add(await FastXsltWorkerClient.StartAsync(
                    workerPath,
                    $"urn:fastxslt:wave-occupancy:source:{index}",
                    source,
                    $"urn:fastxslt:wave-occupancy:stylesheet:{index}",
                    stylesheet));
                Require(await clients[index].TransformAsync($"wave-warm-{index}"), expected);
                nativeClients.Add(NativeFastXsltClient.Create(
                    $"urn:fastxslt:native-wave-occupancy:source:{index}",
                    source,
                    $"urn:fastxslt:native-wave-occupancy:stylesheet:{index}",
                    stylesheet));
                Require(nativeClients[index].Transform($"native-wave-warm-{index}"), expected);
            }

            var measurements = new List<WaveOccupancyMeasurement>(
                rounds * WaveSizes.Length * 2);
            for (var round = 0; round < rounds; round++)
            {
                var sizes = round % 2 == 0 ? WaveSizes : WaveSizes.Reverse();
                foreach (var waveSize in sizes)
                {
                    measurements.Add(await MeasureAsync(
                        clients,
                        waveSize,
                        waves,
                        round + 1,
                        expected));
                    measurements.Add(MeasureNative(
                        nativeClients,
                        waveSize,
                        waves,
                        round + 1,
                        expected));
                }
            }

            return new WaveOccupancyReport(
                workers,
                waves,
                rounds,
                measurements,
                "Each wave contains independent transforms over the same five-item prepared source. Isolated members use batch-of-one transport. Native members use one dedicated thread and handle per worker. Wave size controls request-level fan-out; it does not change source work or transport batch size.");
        }
        finally
        {
            foreach (var client in clients)
            {
                client.Dispose();
            }
            foreach (var client in nativeClients)
            {
                client.Dispose();
            }
        }
    }

    private static async Task<WaveOccupancyMeasurement> MeasureAsync(
        IReadOnlyList<FastXsltWorkerClient> clients,
        int waveSize,
        int waves,
        int round,
        string expected)
    {
        var waveLatencies = new double[waves];
        var active = 0;
        var activeHighWater = 0;
        var started = Stopwatch.StartNew();
        for (var wave = 0; wave < waves; wave++)
        {
            var release = new TaskCompletionSource(
                TaskCreationOptions.RunContinuationsAsynchronously);
            var tasks = new Task[waveSize];
            for (var member = 0; member < waveSize; member++)
            {
                tasks[member] = ExecuteMemberAsync(
                    clients[member],
                    $"wave-{waveSize}-r{round}-w{wave}-m{member}",
                    expected,
                    release.Task,
                    () =>
                    {
                        var observed = Interlocked.Increment(ref active);
                        UpdateHighWater(ref activeHighWater, observed);
                    },
                    () => Interlocked.Decrement(ref active));
            }

            var waveStarted = Stopwatch.GetTimestamp();
            release.SetResult();
            await Task.WhenAll(tasks);
            waveLatencies[wave] = Stopwatch.GetElapsedTime(waveStarted).TotalMicroseconds;
        }
        started.Stop();

        Array.Sort(waveLatencies);
        var members = checked(waveSize * waves);
        return new WaveOccupancyMeasurement(
            "FastXSLT isolated batch-of-one",
            round,
            waveSize,
            waves,
            members,
            activeHighWater,
            started.Elapsed.TotalMilliseconds,
            members / started.Elapsed.TotalSeconds,
            Percentile(waveLatencies, 0.50),
            Percentile(waveLatencies, 0.95),
            Percentile(waveLatencies, 0.99));
    }

    private static WaveOccupancyMeasurement MeasureNative(
        IReadOnlyList<NativeFastXsltClient> clients,
        int waveSize,
        int waves,
        int round,
        string expected)
    {
        using var ready = new CountdownEvent(clients.Count);
        using var release = new Barrier(clients.Count + 1);
        using var completed = new Barrier(clients.Count + 1);
        var waveLatencies = new double[waves];
        var active = 0;
        var activeHighWater = 0;
        Exception? failure = null;
        var threads = clients.Select((client, worker) =>
        {
            var thread = new Thread(() =>
            {
                ready.Signal();
                for (var wave = 0; wave < waves; wave++)
                {
                    release.SignalAndWait();
                    if (worker < waveSize && Volatile.Read(ref failure) is null)
                    {
                        var observed = Interlocked.Increment(ref active);
                        UpdateHighWater(ref activeHighWater, observed);
                        try
                        {
                            Require(
                                client.Transform($"native-wave-{waveSize}-r{round}-w{wave}-m{worker}"),
                                expected);
                        }
                        catch (Exception caught)
                        {
                            Interlocked.CompareExchange(ref failure, caught, null);
                        }
                        finally
                        {
                            Interlocked.Decrement(ref active);
                        }
                    }
                    completed.SignalAndWait();
                }
            })
            {
                IsBackground = true,
                Name = $"fastxslt-native-wave-{worker}"
            };
            thread.Start();
            return thread;
        }).ToArray();

        ready.Wait();
        var started = Stopwatch.StartNew();
        for (var wave = 0; wave < waves; wave++)
        {
            var waveStarted = Stopwatch.GetTimestamp();
            release.SignalAndWait();
            completed.SignalAndWait();
            waveLatencies[wave] = Stopwatch.GetElapsedTime(waveStarted).TotalMicroseconds;
        }
        started.Stop();
        foreach (var thread in threads)
        {
            thread.Join();
        }
        if (failure is not null)
        {
            throw new InvalidOperationException("Native wave execution failed.", failure);
        }

        Array.Sort(waveLatencies);
        var members = checked(waveSize * waves);
        return new WaveOccupancyMeasurement(
            "FastXSLT native dedicated handles",
            round,
            waveSize,
            waves,
            members,
            activeHighWater,
            started.Elapsed.TotalMilliseconds,
            members / started.Elapsed.TotalSeconds,
            Percentile(waveLatencies, 0.50),
            Percentile(waveLatencies, 0.95),
            Percentile(waveLatencies, 0.99));
    }

    private static async Task ExecuteMemberAsync(
        FastXsltWorkerClient client,
        string requestIdentity,
        string expected,
        Task release,
        Action onStart,
        Action onEnd)
    {
        await release;
        onStart();
        try
        {
            Require(await client.TransformAsync(requestIdentity), expected);
        }
        finally
        {
            onEnd();
        }
    }

    private static void Require(string actual, string expected)
    {
        if (!string.Equals(actual, expected, StringComparison.Ordinal))
        {
            throw new InvalidOperationException(
                $"Wave-occupancy result differed. Expected `{expected}`, received `{actual}`.");
        }
    }

    private static void UpdateHighWater(ref int highWater, int active)
    {
        while (true)
        {
            var observed = Volatile.Read(ref highWater);
            if (active <= observed ||
                Interlocked.CompareExchange(ref highWater, active, observed) == observed)
            {
                return;
            }
        }
    }

    private static double Percentile(IReadOnlyList<double> sorted, double percentile)
    {
        var index = Math.Clamp((int)Math.Ceiling(sorted.Count * percentile) - 1, 0, sorted.Count - 1);
        return sorted[index];
    }
}

public sealed record WaveOccupancyReport(
    int WorkerCount,
    int WavesPerMeasurement,
    int Rounds,
    IReadOnlyList<WaveOccupancyMeasurement> Measurements,
    string Scope);

public sealed record WaveOccupancyMeasurement(
    string Engine,
    int Round,
    int WaveSize,
    int Waves,
    int Members,
    int AchievedConcurrencyHighWater,
    double ElapsedMilliseconds,
    double TransformsPerSecond,
    double WaveP50Microseconds,
    double WaveP95Microseconds,
    double WaveP99Microseconds);
