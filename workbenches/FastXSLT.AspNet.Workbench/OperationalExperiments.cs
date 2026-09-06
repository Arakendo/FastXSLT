public static class OperationalExperiments
{
    private static readonly byte[] ReplacementSourceOne =
        "<order><order-item price=\"1.00\" qty=\"1\"/></order>"u8.ToArray();
    private static readonly byte[] ReplacementSourceTwo =
        "<order><order-item price=\"1.00\" qty=\"1\"/><order-item price=\"1.00\" qty=\"1\"/></order>"u8.ToArray();
    private static readonly byte[] UnsupportedMessageStylesheet =
        """<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template match="/"><xsl:message/></xsl:template></xsl:stylesheet>"""u8.ToArray();

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

    public static async Task<object> ExerciseManagedCancellationAsync(
        string workerPath,
        byte[] stylesheet)
    {
        var source = BuildCancellationSource(20_000);
        using var pool = await FastXsltWorkerPool.StartAsync(
            workerPath,
            "urn:fastxslt:managed-cancellation:source",
            source,
            "urn:fastxslt:managed-cancellation:stylesheet",
            stylesheet,
            workers: 1);
        var cancellation = await pool.ExerciseManagedCancellationAsync(
            "managed-cancellation-pre-dispatch",
            "managed-cancellation-active",
            "managed-cancellation-recovery");
        return new
        {
            cancellation,
            sourceItems = 20_000,
            activeOutcome = cancellation.ActiveFailureCode is null ? "completed" : "cancelled",
            completionWinsIfCommittedBeforeSignal = true,
            managedTokenMeansCooperativeRequest = true,
            hardTerminationGuaranteed = false
        };
    }

    public static async Task<object> ExerciseDiagnosticParityAsync(
        string workerPath,
        byte[] source,
        byte[] stylesheet)
    {
        using var client = await FastXsltWorkerClient.StartAsync(
            workerPath,
            "urn:fastxslt:diagnostic:source",
            source,
            "urn:fastxslt:diagnostic:stylesheet",
            stylesheet);
        var processId = client.ProcessId;
        var invalidIdentity = await CaptureFailureAsync(() => client.TransformAsync(""));
        using var cancelled = new CancellationTokenSource();
        cancelled.Cancel();
        var cancellation = await CaptureFailureAsync(
            () => client.TransformAsync("diagnostic-cancelled", cancelled.Token));
        var recoveryResult = await client.TransformAsync("diagnostic-recovery");

        var malformedSource = await CaptureInitializationFailureAsync(
            workerPath,
            "urn:fastxslt:diagnostic:malformed-source",
            "<order></other>"u8.ToArray(),
            "urn:fastxslt:diagnostic:stylesheet",
            stylesheet);
        if (malformedSource.Location != new FastXsltDiagnosticLocation(
            "urn:fastxslt:diagnostic:malformed-source",
            7,
            7))
        {
            throw new InvalidOperationException(
                "Isolated worker did not preserve the structured XML location.");
        }
        var unsupportedStylesheet = await CaptureInitializationFailureAsync(
            workerPath,
            "urn:fastxslt:diagnostic:source",
            source,
            "urn:fastxslt:diagnostic:unsupported-stylesheet",
            UnsupportedMessageStylesheet);
        if (unsupportedStylesheet.Location != new FastXsltDiagnosticLocation(
            "urn:fastxslt:diagnostic:unsupported-stylesheet",
            103,
            117))
        {
            throw new InvalidOperationException(
                "Isolated worker did not preserve the structured stylesheet location.");
        }

        return new
        {
            invalidIdentity,
            malformedSource,
            unsupportedStylesheet,
            cancellation,
            processIdBefore = processId,
            processIdAfter = client.ProcessId,
            recoveryResult,
            sameDiagnosticFieldsAsDirectRustAssertions = true
        };
    }

    public static async Task<object> ExerciseInstructionBudgetAsync(
        string workerPath,
        byte[] source,
        byte[] stylesheet)
    {
        using var client = await FastXsltWorkerClient.StartAsync(
            workerPath,
            "urn:w3c:xslt30:for-004:source",
            source,
            "urn:w3c:xslt30:for-004:stylesheet",
            stylesheet);
        var processId = client.ProcessId;
        var exhaustion = await CaptureFailureAsync(
            () => client.TransformWithXsltInstructionLimitAsync(
                "instruction-budget-exhausted",
                maximumXsltInstructions: 0));
        var recoveryResult = await client.TransformAsync("instruction-budget-recovery");
        return new
        {
            exhaustion,
            configuredMaximumXsltInstructions = 0,
            processIdBefore = processId,
            processIdAfter = client.ProcessId,
            recoveryResult,
            deterministicEngineBudget = true,
            cooperativeCancellation = false,
            workerWasTerminated = false,
            requestWasRetried = false
        };
    }

    public static async Task<object> ExerciseBatchLossClassificationAsync(
        string workerPath,
        byte[] source,
        byte[] stylesheet)
    {
        const int memberCount = 5;
        const string expected = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>36.02</out>";
        var trials = new List<BatchLossTrial>();
        foreach (var parkAtIndex in new[] { 0, 2, 4 })
        {
            var requestIdentities = Enumerable.Range(0, memberCount)
                .Select(index => $"batch-loss-{parkAtIndex}-{index}")
                .ToArray();
            using var client = await FastXsltWorkerClient.StartAsync(
                workerPath,
                "urn:fastxslt:batch-loss:source",
                source,
                "urn:fastxslt:batch-loss:stylesheet",
                stylesheet);
            var processId = client.ProcessId;
            var observations = await client.ReachBatchLossBarrierAsync(
                requestIdentities,
                parkAtIndex);
            var expectedObservationCount = checked(parkAtIndex * 2 + 1);
            if (observations.Count != expectedObservationCount)
            {
                throw new InvalidOperationException(
                    "Batch loss probe did not report the expected sequential prefix.");
            }

            client.TerminateForExperiment();
            var members = requestIdentities.Select((identity, index) =>
            {
                var last = observations.LastOrDefault(value => value.MemberIndex == index);
                return new BatchLossMemberClassification(
                    index,
                    identity,
                    last?.Phase ?? "none",
                    last is null ? "unstarted" : "operationally-ambiguous",
                    CorrelatedOutcomeTransferred: false);
            }).ToArray();
            trials.Add(new BatchLossTrial(
                parkAtIndex,
                processId,
                observations,
                members));
        }

        const int truncateAtIndex = 2;
        var transferRequestIdentities = Enumerable.Range(0, memberCount)
            .Select(index => $"batch-transfer-loss-{index}")
            .ToArray();
        BatchTransferLossTrial transferLoss;
        using (var client = await FastXsltWorkerClient.StartAsync(
            workerPath,
            "urn:fastxslt:batch-loss:source",
            source,
            "urn:fastxslt:batch-loss:stylesheet",
            stylesheet))
        {
            var processId = client.ProcessId;
            var probe = await client.ReachBatchTransferLossBarrierAsync(
                transferRequestIdentities,
                truncateAtIndex);
            if (probe.CompleteOutcomes.Count != truncateAtIndex ||
                probe.CompleteOutcomes.Any(outcome => outcome.Failure is not null) ||
                probe.CompleteOutcomes.Any(outcome => outcome.Result != expected) ||
                probe.ObservedResultBytes <= 0 ||
                probe.ObservedResultBytes >= probe.DeclaredResultBytes)
            {
                throw new InvalidOperationException(
                    "Batch transfer-loss probe did not expose the expected complete prefix and partial member.");
            }
            client.TerminateForExperiment();
            var members = transferRequestIdentities.Select((identity, index) =>
                new BatchLossMemberClassification(
                    index,
                    identity,
                    index < truncateAtIndex
                        ? "correlated-outcome-transferred"
                        : index == truncateAtIndex ? "partial-result-frame" : "none",
                    index < truncateAtIndex
                        ? "complete"
                        : index == truncateAtIndex ? "operationally-ambiguous" : "unstarted",
                    CorrelatedOutcomeTransferred: index < truncateAtIndex)).ToArray();
            transferLoss = new BatchTransferLossTrial(
                truncateAtIndex,
                processId,
                probe.CompleteOutcomes.Count,
                probe.PartialRequestIdentity,
                probe.DeclaredResultBytes,
                probe.ObservedResultBytes,
                members);
        }

        int malformedProcessId;
        int malformedExitCode;
        using (var malformed = await FastXsltWorkerClient.StartAsync(
            workerPath,
            "urn:fastxslt:batch-loss:source",
            source,
            "urn:fastxslt:batch-loss:stylesheet",
            stylesheet))
        {
            malformedProcessId = malformed.ProcessId;
            malformedExitCode = await malformed.SendTruncatedBatchCommandForExperimentAsync();
        }

        var unacknowledgedIdentities = Enumerable.Range(0, memberCount)
            .Select(index => $"batch-unacknowledged-{index}")
            .ToArray();
        int unacknowledgedProcessId;
        using (var unacknowledged = await FastXsltWorkerClient.StartAsync(
            workerPath,
            "urn:fastxslt:batch-loss:source",
            source,
            "urn:fastxslt:batch-loss:stylesheet",
            stylesheet))
        {
            unacknowledgedProcessId = unacknowledged.ProcessId;
            await unacknowledged.DispatchBatchWithoutObservationForExperimentAsync(
                unacknowledgedIdentities);
            unacknowledged.TerminateForExperiment();
        }

        var activeCancellationIdentities = Enumerable.Range(0, memberCount)
            .Select(index => $"batch-active-cancellation-{index}")
            .ToArray();
        IReadOnlyList<IsolatedWorkerBatchOutcome> activeCancellationOutcomes;
        string activeCancellationRecovery;
        using (var activeCancellation = await FastXsltWorkerClient.StartAsync(
            workerPath,
            "urn:fastxslt:batch-loss:source",
            source,
            "urn:fastxslt:batch-loss:stylesheet",
            stylesheet))
        {
            activeCancellationOutcomes = await activeCancellation.TransformActiveCancellationBatchAsync(
                activeCancellationIdentities,
                cancelAtIndex: 2);
            activeCancellationRecovery = await activeCancellation.TransformAsync(
                "batch-active-cancellation-recovery");
        }

        using var replacement = await FastXsltWorkerClient.StartAsync(
            workerPath,
            "urn:fastxslt:batch-loss:source",
            source,
            "urn:fastxslt:batch-loss:stylesheet",
            stylesheet);
        var recovery = await replacement.TransformAsync("batch-loss-recovery");
        return new
        {
            trials,
            transferLoss,
            malformedCommand = new
            {
                processId = malformedProcessId,
                exitCode = malformedExitCode,
                fullCommandDecoded = false,
                memberAttemptsAdmitted = 0,
                disposition = "unstarted",
                memberAttemptsRetried = false
            },
            unacknowledgedDispatch = new
            {
                processId = unacknowledgedProcessId,
                memberCount,
                acknowledgementObserved = false,
                correlatedOutcomesObserved = 0,
                disposition = "operationally-ambiguous",
                memberAttemptsRetried = false
            },
            activeCancellation = new
            {
                cancelAtIndex = 2,
                outcomes = activeCancellationOutcomes.Select(outcome => new
                {
                    outcome.RequestIdentity,
                    outcome.Result,
                    failureCode = outcome.Failure?.Code,
                    failureCategory = outcome.Failure?.Category
                }),
                recovery = activeCancellationRecovery,
                oneSequentialExecutionLane = true,
                laterSiblingsExecuted = true
            },
            recovery,
            aggregateResponseOracle = true,
            memberResultsTransferredBeforeLoss = 0,
            killedMemberAttemptsRetried = false,
            finishedButUntransferredIsAmbiguous = true,
            laterMembersAreUnstarted = true
        };
    }

    public static async Task<object> MeasureBatchNaturalCancellationRacesAsync(
        string workerPath,
        byte[] stylesheet)
    {
        const int trials = 25;
        const string expected = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>20000.00</out>";
        var source = BuildCancellationSource(20_000);
        using var client = await FastXsltWorkerClient.StartAsync(
            workerPath,
            "urn:fastxslt:batch-natural-cancellation:source",
            source,
            "urn:fastxslt:batch-natural-cancellation:stylesheet",
            stylesheet);
        var cancellations = 0;
        var completions = 0;
        var invalidTargetOutcomes = 0;
        var siblingFailures = 0;
        for (var trial = 0; trial < trials; trial++)
        {
            var identities = new[]
            {
                $"batch-natural-{trial}-before",
                $"batch-natural-{trial}-target",
                $"batch-natural-{trial}-after"
            };
            var delay = trial < 10
                ? TimeSpan.Zero
                : trial < 20 ? TimeSpan.FromMilliseconds(1) : TimeSpan.FromMilliseconds(5);
            var outcomes = await client.TransformNaturalCancellationBatchAsync(
                identities,
                cancelAtIndex: 1,
                delay);
            if (outcomes[0].Result != expected || outcomes[2].Result != expected)
            {
                siblingFailures++;
            }
            var target = outcomes[1];
            if (target.Result == expected)
            {
                completions++;
            }
            else if (target.Failure is { Code: "FXCT0001", Category: "cancelled" })
            {
                cancellations++;
            }
            else
            {
                invalidTargetOutcomes++;
            }
        }
        var recovery = await client.TransformAsync("batch-natural-cancellation-recovery");
        return new
        {
            trials,
            cancellations,
            completions,
            invalidTargetOutcomes,
            siblingFailures,
            recovery,
            completionWinsIfCommittedBeforeSignal = true,
            firstChargeBarrierUsed = false,
            oneSequentialExecutionLane = true
        };
    }

    public static async Task<object> ExerciseNativeBoundaryAsync(
        byte[] source,
        byte[] stylesheet)
    {
        using var client = NativeFastXsltClient.Create(
            "urn:fastxslt:native-boundary:source",
            source,
            "urn:fastxslt:native-boundary:stylesheet",
            stylesheet);
        NativeFastXsltException invalidIdentity;
        try
        {
            _ = client.Transform("");
            throw new InvalidOperationException("Empty native request identity unexpectedly succeeded.");
        }
        catch (NativeFastXsltException failure)
        {
            invalidIdentity = failure;
        }
        var recoveryResult = client.Transform("native-boundary-recovery");

        NativeFastXsltException cancellation;
        try
        {
            _ = client.TransformWithInvocationPolicy(
                "native-controlled-cancelled",
                cancellationRequested: true,
                maximumXsltInstructions: 1_000_000);
            throw new InvalidOperationException("Pre-signalled native cancellation unexpectedly succeeded.");
        }
        catch (NativeFastXsltException failure)
        {
            cancellation = failure;
        }

        NativeFastXsltException instructionBudget;
        try
        {
            _ = client.TransformWithInvocationPolicy(
                "native-instruction-budget",
                cancellationRequested: false,
                maximumXsltInstructions: 0);
            throw new InvalidOperationException("Zero native instruction budget unexpectedly succeeded.");
        }
        catch (NativeFastXsltException failure)
        {
            instructionBudget = failure;
        }
        var controlledRecoveryResult = client.Transform("native-controlled-recovery");

        NativeFastXsltException malformedSource;
        try
        {
            using var unexpected = NativeFastXsltClient.Create(
                "urn:fastxslt:native-boundary:malformed-source",
                "<order></other>"u8.ToArray(),
                "urn:fastxslt:native-boundary:stylesheet",
                stylesheet);
            throw new InvalidOperationException("Malformed native source unexpectedly initialized.");
        }
        catch (NativeFastXsltException failure)
        {
            malformedSource = failure;
        }

        NativeFastXsltException unsupportedStylesheet;
        try
        {
            using var unexpected = NativeFastXsltClient.Create(
                "urn:fastxslt:native-boundary:unsupported-source",
                source,
                "urn:fastxslt:diagnostic:unsupported-stylesheet",
                UnsupportedMessageStylesheet);
            throw new InvalidOperationException("Unsupported native stylesheet unexpectedly initialized.");
        }
        catch (NativeFastXsltException failure)
        {
            unsupportedStylesheet = failure;
        }

        using var first = NativeFastXsltClient.Create(
            "urn:fastxslt:native-boundary:concurrent-source-1",
            source,
            "urn:fastxslt:native-boundary:concurrent-stylesheet-1",
            stylesheet);
        using var second = NativeFastXsltClient.Create(
            "urn:fastxslt:native-boundary:concurrent-source-2",
            source,
            "urn:fastxslt:native-boundary:concurrent-stylesheet-2",
            stylesheet);
        var concurrentResults = await Task.WhenAll(
            Task.Run(() => first.Transform("native-concurrent-1")),
            Task.Run(() => second.Transform("native-concurrent-2")));

        var disposed = NativeFastXsltClient.Create(
            "urn:fastxslt:native-boundary:dispose-source",
            source,
            "urn:fastxslt:native-boundary:dispose-stylesheet",
            stylesheet);
        disposed.Dispose();
        disposed.Dispose();
        var useAfterDisposeRejected = false;
        try
        {
            _ = disposed.Transform("native-after-dispose");
        }
        catch (ObjectDisposedException)
        {
            useAfterDisposeRejected = true;
        }

        return new
        {
            invalidIdentity = new DiagnosticEvidence(
                invalidIdentity.Code,
                invalidIdentity.Category,
                invalidIdentity.RequestId,
                invalidIdentity.Location,
                invalidIdentity.Detail),
            malformedSource = new DiagnosticEvidence(
                malformedSource.Code,
                malformedSource.Category,
                malformedSource.RequestId,
                malformedSource.Location,
                malformedSource.Detail),
            unsupportedStylesheet = new DiagnosticEvidence(
                unsupportedStylesheet.Code,
                unsupportedStylesheet.Category,
                unsupportedStylesheet.RequestId,
                unsupportedStylesheet.Location,
                unsupportedStylesheet.Detail),
            cancellation = new DiagnosticEvidence(
                cancellation.Code,
                cancellation.Category,
                cancellation.RequestId,
                cancellation.Location,
                cancellation.Detail),
            instructionBudget = new DiagnosticEvidence(
                instructionBudget.Code,
                instructionBudget.Category,
                instructionBudget.RequestId,
                instructionBudget.Location,
                instructionBudget.Detail),
            recoveryResult,
            controlledRecoveryResult,
            concurrentResults,
            independentHandlesExecutedConcurrently = true,
            controlsWereScalarAndPreDispatch = true,
            activeMidExecutionSignalSupported = false,
            hardTerminationGuaranteed = false,
            doubleDisposeWasIdempotent = true,
            useAfterDisposeRejected
        };
    }

    private sealed record BatchLossTrial(
        int ParkAtIndex,
        int ProcessId,
        IReadOnlyList<IsolatedBatchLossObservation> Observations,
        IReadOnlyList<BatchLossMemberClassification> Members);

    private sealed record BatchLossMemberClassification(
        int MemberIndex,
        string RequestIdentity,
        string LastObservedPhase,
        string Disposition,
        bool CorrelatedOutcomeTransferred);

    private sealed record BatchTransferLossTrial(
        int TruncateAtIndex,
        int ProcessId,
        int CompleteOutcomeCount,
        string PartialRequestIdentity,
        int DeclaredResultBytes,
        int ObservedResultBytes,
        IReadOnlyList<BatchLossMemberClassification> Members);

    public static async Task<object> ExerciseNativeGenerationReplacementAsync(
        byte[] stylesheet)
    {
        using var host = NativeFastXsltGenerationHost.Create(
            "native-generation-001",
            "urn:fastxslt:native-generation:source:g1",
            ReplacementSourceOne,
            "urn:fastxslt:native-generation:stylesheet:g1",
            stylesheet,
            engines: 1);
        using var oldLease = host.AcquireCurrent();
        var oldPool = oldLease.Pool;
        var retiredIdentity = host.Replace(
            "native-generation-002",
            "urn:fastxslt:native-generation:source:g2",
            ReplacementSourceTwo,
            "urn:fastxslt:native-generation:stylesheet:g2",
            stylesheet,
            engines: 1);
        var newGeneration = await host.TransformAsync("native-replacement-new");
        var oldResult = await oldPool.TransformAsync("native-replacement-old-in-flight");
        oldLease.Dispose();
        var oldGenerationDisposedAfterLeaseRelease = false;
        try
        {
            _ = await oldPool.TransformAsync("native-replacement-after-drain");
        }
        catch (ObjectDisposedException)
        {
            oldGenerationDisposedAfterLeaseRelease = true;
        }
        return new
        {
            retiredGenerationIdentity = retiredIdentity,
            oldLeaseGenerationIdentity = oldLease.Identity,
            oldRequestIdentity = "native-replacement-old-in-flight",
            oldResult,
            newGeneration,
            replacementInitializedBeforePromotion = true,
            promotionWasExplicit = true,
            oldGenerationDisposedAfterLeaseRelease
        };
    }

    public static async Task<object> ExerciseNativeActiveCancellationAsync(
        byte[] stylesheet)
    {
        using var client = NativeFastXsltClient.Create(
            "urn:fastxslt:native-active:source",
            BuildCancellationSource(20_000),
            "urn:fastxslt:native-active:stylesheet",
            stylesheet);
        var observation = await client.ExerciseActiveCancellationAsync(
            "native-active-cancelled",
            observationTimeout: TimeSpan.FromSeconds(5));
        var recoveryResult = client.Transform("native-active-recovery");
        return new
        {
            cancellation = new DiagnosticEvidence(
                observation.Failure.Code,
                observation.Failure.Category,
                observation.Failure.RequestId,
                observation.Failure.Location,
                observation.Failure.Detail),
            observation.SignalToObservationMilliseconds,
            observation.FirstChargeObserved,
            observation.UnrelatedSignalIgnored,
            observation.ControlDoubleDisposeWasIdempotent,
            recoveryResult,
            sourceItems = 20_000,
            cancellationWasCooperative = true,
            completionWinsIfCommittedBeforeSignal = true,
            hardTerminationGuaranteed = false,
            firstChargeBarrierWasExperimental = true
        };
    }

    public static async Task<object> MeasureNativeNaturalCancellationRacesAsync(
        byte[] stylesheet)
    {
        const int trials = 25;
        const string expected = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>20000.00</out>";
        using var client = NativeFastXsltClient.Create(
            "urn:fastxslt:native-natural:source",
            BuildCancellationSource(20_000),
            "urn:fastxslt:native-natural:stylesheet",
            stylesheet);
        var cancellations = 0;
        var completions = 0;
        var cancellationLatencies = new List<double>();
        var observedChargeDetails = new HashSet<string>(StringComparer.Ordinal);
        for (var trial = 0; trial < trials; trial++)
        {
            using var cancellation = new CancellationTokenSource();
            var invocation = client.TransformAsync(
                $"native-natural-{trial}",
                cancellation.Token);
            await Task.Yield();
            var signal = System.Diagnostics.Stopwatch.StartNew();
            cancellation.Cancel();
            try
            {
                var result = await invocation;
                if (!StringComparer.Ordinal.Equals(result, expected))
                {
                    throw new InvalidOperationException("Native natural-race completion changed result semantics.");
                }
                completions++;
            }
            catch (NativeFastXsltException failure)
            {
                signal.Stop();
                if (failure.Code != "FXCT0001" ||
                    failure.Category != "cancelled" ||
                    failure.RequestId != $"native-natural-{trial}" ||
                    !failure.Detail.StartsWith(
                        "host cancellation observed while charging ",
                        StringComparison.Ordinal))
                {
                    throw new InvalidOperationException("Native natural-race cancellation changed diagnostic semantics.");
                }
                cancellations++;
                cancellationLatencies.Add(signal.Elapsed.TotalMilliseconds);
                observedChargeDetails.Add(failure.Detail);
            }
        }
        cancellationLatencies.Sort();
        var recoveryResult = client.Transform("native-natural-recovery");
        return new
        {
            trials,
            cancellations,
            completions,
            minimumCancellationMilliseconds = cancellationLatencies.Count == 0
                ? (double?)null
                : cancellationLatencies[0],
            medianCancellationMilliseconds = cancellationLatencies.Count == 0
                ? (double?)null
                : cancellationLatencies[cancellationLatencies.Count / 2],
            maximumCancellationMilliseconds = cancellationLatencies.Count == 0
                ? (double?)null
                : cancellationLatencies[^1],
            observedChargeDetails = observedChargeDetails.Order().ToArray(),
            recoveryResult,
            sourceItems = 20_000,
            firstChargeBarrierUsed = false,
            managedCancellationTokenAdapted = true,
            diagnosticFieldsValidated = true,
            completionWinsIfCommittedBeforeSignal = true,
            hardTerminationGuaranteed = false
        };
    }

    private static async Task<DiagnosticEvidence> CaptureInitializationFailureAsync(
        string workerPath,
        string sourceIdentity,
        byte[] source,
        string stylesheetIdentity,
        byte[] stylesheet) => await CaptureFailureAsync(async () =>
        {
            using var unexpected = await FastXsltWorkerClient.StartAsync(
                workerPath,
                sourceIdentity,
                source,
                stylesheetIdentity,
                stylesheet);
            return "unexpected initialization";
        });

    private static async Task<DiagnosticEvidence> CaptureFailureAsync(
        Func<Task<string>> operation)
    {
        try
        {
            _ = await operation();
            throw new InvalidOperationException("Diagnostic probe unexpectedly succeeded.");
        }
        catch (FastXsltWorkerException failure)
        {
            return new DiagnosticEvidence(
                failure.Code,
                failure.Category,
                failure.RequestId,
                failure.Location,
                failure.Detail);
        }
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

public sealed record DiagnosticEvidence(
    string Code,
    string Category,
    string? RequestId,
    FastXsltDiagnosticLocation? Location,
    string Detail);
