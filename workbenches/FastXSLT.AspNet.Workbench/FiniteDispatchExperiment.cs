using System.Diagnostics;
using System.Text;

public static class FiniteDispatchExperiment
{
    private static readonly int[] JobCounts = [8, 32, 128, 500, 4_096, 50_000];
    private static readonly int[] ClaimSizes = [1, 2, 4, 8];

    public static FiniteDispatchReport Run(
        byte[] source,
        byte[] stylesheet,
        int rounds)
    {
        rounds = Math.Clamp(rounds, 1, 9);
        const int workers = 8;
        const string expected = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>36.02</out>";
        var clients = new List<NativeFastXsltClient>(workers);
        try
        {
            for (var index = 0; index < workers; index++)
            {
                clients.Add(NativeFastXsltClient.Create(
                    $"urn:fastxslt:finite-dispatch:source:{index}",
                    source,
                    $"urn:fastxslt:finite-dispatch:stylesheet:{index}",
                    stylesheet));
                Require(clients[index].Transform($"finite-warm-{index}"), expected);
            }
            var transforms = clients
                .Select(client => new Func<int, string, string>(
                    (_, identity) => client.Transform(identity)))
                .ToArray();

            var measurements = new List<FiniteDispatchMeasurement>(
                rounds * JobCounts.Length * (ClaimSizes.Length + 1));
            for (var round = 0; round < rounds; round++)
            {
                foreach (var jobs in JobCounts)
                {
                    var jobKinds = new int[jobs];
                    var candidates = new List<(string Name, int ClaimSize)>
                    {
                        ("static-balanced", 0)
                    };
                    candidates.AddRange(ClaimSizes.Select(size => ("completion-driven", size)));
                    if ((round + jobs) % 2 != 0)
                    {
                        candidates.Reverse();
                    }
                    foreach (var candidate in candidates)
                    {
                        measurements.Add(Measure(
                            transforms,
                            jobKinds,
                            round + 1,
                            "uniform-items-5",
                            candidate.Name,
                            candidate.ClaimSize,
                            [expected]));
                    }
                }
            }

            return new FiniteDispatchReport(
                workers,
                rounds,
                JobCounts,
                measurements,
                "Native-only AR-0022 reference. Source work is fixed. Static balanced assignment and completion-driven terminal-refill claims are private experiment topologies; claim size is not a host setting or default.");
        }
        finally
        {
            foreach (var client in clients)
            {
                client.Dispose();
            }
        }
    }

    public static FiniteDispatchReport RunMixed(
        byte[] stylesheet,
        int rounds)
    {
        rounds = Math.Clamp(rounds, 1, 9);
        const int workers = 8;
        const int jobs = 4_096;
        int[] items = [5, 50, 500];
        var expected = items
            .Select(itemCount =>
                $"<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>{itemCount}.00</out>")
            .ToArray();
        var clients = new List<NativeFastXsltClient>(workers * items.Length);
        try
        {
            var transforms = new List<Func<int, string, string>>(workers);
            for (var worker = 0; worker < workers; worker++)
            {
                var workerClients = new NativeFastXsltClient[items.Length];
                for (var kind = 0; kind < items.Length; kind++)
                {
                    workerClients[kind] = NativeFastXsltClient.Create(
                        $"urn:fastxslt:finite-dispatch:mixed:source:{worker}:{items[kind]}",
                        BuildSource(items[kind]),
                        $"urn:fastxslt:finite-dispatch:mixed:stylesheet:{worker}:{items[kind]}",
                        stylesheet);
                    clients.Add(workerClients[kind]);
                    Require(
                        workerClients[kind].Transform($"finite-mixed-warm-{worker}-{kind}"),
                        expected[kind]);
                }
                transforms.Add((kind, identity) => workerClients[kind].Transform(identity));
            }

            var workloads = new (string Name, int[] JobKinds)[]
            {
                (
                    "mixed-interleaved-5-50-500",
                    Enumerable.Range(0, jobs).Select(index => index % items.Length).ToArray()),
                (
                    "mixed-clustered-5-50-500",
                    Enumerable.Range(0, jobs)
                        .Select(index => Math.Min(items.Length - 1, index * items.Length / jobs))
                        .ToArray())
            };
            var measurements = new List<FiniteDispatchMeasurement>(
                rounds * workloads.Length * (ClaimSizes.Length + 1));
            for (var round = 0; round < rounds; round++)
            {
                foreach (var workload in workloads)
                {
                    var candidates = new List<(string Name, int ClaimSize)>
                    {
                        ("static-balanced", 0)
                    };
                    candidates.AddRange(ClaimSizes.Select(size => ("completion-driven", size)));
                    if ((round + workload.Name.Length) % 2 != 0)
                    {
                        candidates.Reverse();
                    }
                    foreach (var candidate in candidates)
                    {
                        measurements.Add(Measure(
                            transforms,
                            workload.JobKinds,
                            round + 1,
                            workload.Name,
                            candidate.Name,
                            candidate.ClaimSize,
                            expected));
                    }
                }
            }

            return new FiniteDispatchReport(
                workers,
                rounds,
                [jobs],
                measurements,
                "Native-only AR-0022 mixed-duration pressure. Every set contains the same 5/50/500-item population in interleaved or clustered order. Result size stays small; mixed-result pressure remains separate.");
        }
        finally
        {
            foreach (var client in clients)
            {
                client.Dispose();
            }
        }
    }

    private static FiniteDispatchMeasurement Measure(
        IReadOnlyList<Func<int, string, string>> transforms,
        IReadOnlyList<int> jobKinds,
        int round,
        string workload,
        string topology,
        int claimSize,
        IReadOnlyList<string> expected)
    {
        var jobs = jobKinds.Count;
        var identities = Enumerable.Range(0, jobs)
            .Select(index => $"finite-{workload}-{topology}-c{claimSize}-r{round}-j{jobs}-{index}")
            .ToArray();
        using var release = new Barrier(transforms.Count + 1);
        var busyTicks = new long[transforms.Count];
        var claimTicks = new long[transforms.Count];
        var firstStarted = new long[transforms.Count];
        var lastFinished = new long[transforms.Count];
        var completedByWorker = new int[transforms.Count];
        var active = 0;
        var activeHighWater = 0;
        var nextJob = 0;
        var completedJobs = 0;
        var acquisitionCount = 0;
        Exception? failure = null;

        var threads = transforms.Select((transform, worker) =>
        {
            var thread = new Thread(() =>
            {
                release.SignalAndWait();
                if (topology == "static-balanced")
                {
                    var first = worker * jobs / transforms.Count;
                    var end = (worker + 1) * jobs / transforms.Count;
                    ExecuteRange(first, end);
                }
                else
                {
                    while (Volatile.Read(ref failure) is null)
                    {
                        var claimed = Stopwatch.GetTimestamp();
                        var first = Interlocked.Add(ref nextJob, claimSize) - claimSize;
                        claimTicks[worker] += Stopwatch.GetTimestamp() - claimed;
                        if (first >= jobs)
                        {
                            break;
                        }
                        Interlocked.Increment(ref acquisitionCount);
                        ExecuteRange(first, Math.Min(first + claimSize, jobs));
                    }
                }

                void ExecuteRange(int first, int end)
                {
                    for (var job = first; job < end && Volatile.Read(ref failure) is null; job++)
                    {
                        var started = Stopwatch.GetTimestamp();
                        if (firstStarted[worker] == 0)
                        {
                            firstStarted[worker] = started;
                        }
                        var observed = Interlocked.Increment(ref active);
                        UpdateHighWater(ref activeHighWater, observed);
                        try
                        {
                            var kind = jobKinds[job];
                            Require(transform(kind, identities[job]), expected[kind]);
                        }
                        catch (Exception caught)
                        {
                            Interlocked.CompareExchange(ref failure, caught, null);
                        }
                        finally
                        {
                            var finished = Stopwatch.GetTimestamp();
                            busyTicks[worker] += finished - started;
                            lastFinished[worker] = finished;
                            completedByWorker[worker]++;
                            Interlocked.Increment(ref completedJobs);
                            Interlocked.Decrement(ref active);
                        }
                    }
                }
            })
            {
                IsBackground = true,
                Name = $"fastxslt-finite-dispatch-{worker}"
            };
            thread.Start();
            return thread;
        }).ToArray();

        var wallStarted = Stopwatch.GetTimestamp();
        release.SignalAndWait();
        foreach (var thread in threads)
        {
            thread.Join();
        }
        var wallFinished = Stopwatch.GetTimestamp();

        if (failure is not null)
        {
            throw new InvalidOperationException("Finite dispatch execution failed.", failure);
        }
        if (completedJobs != jobs)
        {
            throw new InvalidOperationException(
                $"Finite dispatch completed {completedJobs} of {jobs} jobs.");
        }

        var participatingFirst = firstStarted.Where(value => value != 0).ToArray();
        var participatingLast = lastFinished.Where(value => value != 0).ToArray();
        var elapsedTicks = participatingLast.Max() - wallStarted;
        var fillTicks = participatingFirst.Max() - wallStarted;
        var tailTicks = participatingLast.Max() - participatingLast.Min();
        var busyTotal = busyTicks.Sum();
        var claimTotal = claimTicks.Sum();
        return new FiniteDispatchMeasurement(
            round,
            workload,
            topology,
            claimSize,
            jobs,
            transforms.Count,
            participatingFirst.Length,
            activeHighWater,
            acquisitionCount,
            completedByWorker.Min(),
            completedByWorker.Max(),
            ToMilliseconds(elapsedTicks),
            jobs / (elapsedTicks / (double)Stopwatch.Frequency),
            ToMicroseconds(fillTicks),
            ToMicroseconds(tailTicks),
            elapsedTicks == 0 ? 0 : busyTotal / (double)(elapsedTicks * transforms.Count),
            ToMicroseconds(claimTotal),
            wallFinished >= participatingLast.Max()
                ? ToMicroseconds(wallFinished - participatingLast.Max())
                : 0);
    }

    private static void Require(string actual, string expected)
    {
        if (!string.Equals(actual, expected, StringComparison.Ordinal))
        {
            throw new InvalidOperationException(
                $"Finite dispatch result differed. Expected `{expected}`, received `{actual}`.");
        }
    }

    private static byte[] BuildSource(int items)
    {
        var source = new StringBuilder("<?xml version=\"1.0\"?><order>");
        for (var index = 0; index < items; index++)
        {
            source.Append("<order-item price=\"1.00\" qty=\"1\"/>");
        }
        source.Append("</order>");
        return Encoding.UTF8.GetBytes(source.ToString());
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

    private static double ToMicroseconds(long ticks) =>
        ticks * 1_000_000.0 / Stopwatch.Frequency;

    private static double ToMilliseconds(long ticks) =>
        ticks * 1_000.0 / Stopwatch.Frequency;
}

public sealed record FiniteDispatchReport(
    int Workers,
    int Rounds,
    IReadOnlyList<int> JobCounts,
    IReadOnlyList<FiniteDispatchMeasurement> Measurements,
    string Scope);

public sealed record FiniteDispatchMeasurement(
    int Round,
    string Workload,
    string Topology,
    int ClaimSize,
    int Jobs,
    int Workers,
    int ParticipatingWorkers,
    int ActiveHighWater,
    int QueueAcquisitions,
    int MinimumJobsPerWorker,
    int MaximumJobsPerWorker,
    double ElapsedMilliseconds,
    double TransformsPerSecond,
    double FillMicroseconds,
    double TailDrainMicroseconds,
    double WorkerBusyFraction,
    double AggregateClaimMicroseconds,
    double JoinAfterLastCompletionMicroseconds);
