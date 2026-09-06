using System.Text;

public static class IncrementalBatchOperationalExperiment
{
    public static async Task<object> ExerciseConsumerBoundaryAsync(string workerPath)
    {
        const int items = 5_000;
        var (stylesheet, expected) = BuildFixture(items);
        var identities = Enumerable.Range(0, 4)
            .Select(index => $"incremental-consumer-{index}")
            .ToArray();
        using var worker = await FastXsltWorkerClient.StartAsync(
            workerPath,
            "urn:fastxslt:incremental-consumer:source",
            "<root/>"u8.ToArray(),
            "urn:fastxslt:incremental-consumer:stylesheet",
            stylesheet);
        var unsupportedProtocol =
            await worker.ProbeUnsupportedIncrementalBatchProtocolVersionAsync();
        var unsupportedProtocolRecovery = await worker.TransformAsync(
            "incremental-protocol-recovery");
        var callbackCount = 0;
        var callbackResultsAreExact = true;
        var result = await worker.ConsumeIncrementalBatchAsync(
            identities,
            outcome =>
            {
                callbackResultsAreExact &= outcome.Failure is null &&
                    StringComparer.Ordinal.Equals(
                        outcome.RequestIdentity,
                        identities[callbackCount]) &&
                    StringComparer.Ordinal.Equals(outcome.Result, expected);
                callbackCount++;
                return ValueTask.CompletedTask;
            });
        var expectedRequestBytes =
            1 + sizeof(uint) + sizeof(int) + sizeof(int) + identities.Sum(identity =>
                sizeof(int) + Encoding.UTF8.GetByteCount(identity) + 2);
        var expectedOutcomeWireBytes = identities
            .Select(identity =>
                1 + sizeof(uint) + 1 +
                sizeof(int) + Encoding.UTF8.GetByteCount(identity) +
                sizeof(int) + Encoding.UTF8.GetByteCount(expected))
            .ToArray();
        var expectedResponseBytes =
            1 + sizeof(uint) + sizeof(uint) +
            1 + sizeof(uint) +
            expectedOutcomeWireBytes.Sum();
        var retainingResult = await worker.TransformIncrementalBatchAsync(identities);
        var retainingResultsAreExact = retainingResult.Outcomes
            .Select((outcome, index) =>
                outcome.Failure is null &&
                StringComparer.Ordinal.Equals(outcome.RequestIdentity, identities[index]) &&
                StringComparer.Ordinal.Equals(outcome.Result, expected))
            .All(static exact => exact);
        var recovery = await worker.TransformAsync("incremental-consumer-recovery");

        const int abandonAfter = 2;
        var abandonedCallbackCount = 0;
        var abandonedWorkerRejectedReuse = false;
        using (var abandonedWorker = await FastXsltWorkerClient.StartAsync(
            workerPath,
            "urn:fastxslt:incremental-abandon:source",
            "<root/>"u8.ToArray(),
            "urn:fastxslt:incremental-abandon:stylesheet",
            stylesheet))
        {
            try
            {
                _ = await abandonedWorker.ConsumeIncrementalBatchAsync(
                    identities,
                    _ =>
                    {
                        abandonedCallbackCount++;
                        if (abandonedCallbackCount == abandonAfter)
                        {
                            throw new ConsumerAbandonedIncrementalBatchException();
                        }
                        return ValueTask.CompletedTask;
                    });
                throw new InvalidOperationException(
                    "Incremental consumer abandonment unexpectedly completed.");
            }
            catch (ConsumerAbandonedIncrementalBatchException)
            {
            }

            try
            {
                _ = await abandonedWorker.TransformAsync("abandoned-worker-reuse");
            }
            catch (IOException)
            {
                abandonedWorkerRejectedReuse = true;
            }
        }

        var lossCallbackCount = 0;
        IsolatedWorkerIncrementalBatchTransportException transportLoss;
        using (var lostWorker = await FastXsltWorkerClient.StartAsync(
            workerPath,
            "urn:fastxslt:incremental-loss:source",
            "<root/>"u8.ToArray(),
            "urn:fastxslt:incremental-loss:stylesheet",
            stylesheet))
        {
            try
            {
                _ = await lostWorker.ConsumeIncrementalBatchAsync(
                    identities,
                    _ =>
                    {
                        lossCallbackCount++;
                        if (lossCallbackCount == 1)
                        {
                            lostWorker.TerminateForExperiment();
                        }
                        return ValueTask.CompletedTask;
                    });
                throw new InvalidOperationException(
                    "Incremental worker loss unexpectedly completed.");
            }
            catch (IsolatedWorkerIncrementalBatchTransportException failure)
            {
                transportLoss = failure;
            }
        }

        using var replacement = await FastXsltWorkerClient.StartAsync(
            workerPath,
            "urn:fastxslt:incremental-replacement:source",
            "<root/>"u8.ToArray(),
            "urn:fastxslt:incremental-replacement:stylesheet",
            stylesheet);
        var replacementResult = await replacement.TransformAsync(
            "incremental-consumer-replacement");
        return new
        {
            memberCount = identities.Length,
            unsupportedProtocolCode = unsupportedProtocol.Code,
            unsupportedProtocolCategory = unsupportedProtocol.Category,
            unsupportedProtocolRecoveryIsExact = StringComparer.Ordinal.Equals(
                unsupportedProtocolRecovery,
                expected),
            callbackCount,
            callbackResultsAreExact,
            streamedCompleteOutcomeCount = result.CompleteOutcomeCount,
            retainedOutcomeCount = result.Outcomes.Count,
            result.EncodedRequestBytes,
            result.EncodedResponseBytes,
            expectedRequestBytes,
            expectedResponseBytes,
            result.PeakAdapterRetainedOutcomeCount,
            result.PeakAdapterRetainedOutcomeWireBytes,
            expectedPeakAdapterRetainedOutcomeWireBytes = expectedOutcomeWireBytes.Max(),
            retainingResultsAreExact,
            retainingOutcomeCount = retainingResult.Outcomes.Count,
            retainingPeakAdapterRetainedOutcomeCount =
                retainingResult.PeakAdapterRetainedOutcomeCount,
            retainingPeakAdapterRetainedOutcomeWireBytes =
                retainingResult.PeakAdapterRetainedOutcomeWireBytes,
            expectedRetainingOutcomeWireBytes = expectedOutcomeWireBytes.Sum(),
            firstOutcomeMilliseconds = result.FirstOutcomeElapsed.TotalMilliseconds,
            finalOutcomeMilliseconds = result.FinalOutcomeElapsed.TotalMilliseconds,
            recoveryIsExact = StringComparer.Ordinal.Equals(recovery, expected),
            abandonedCallbackCount,
            abandonedWorkerRejectedReuse,
            lossCallbackCount,
            transportLoss.Acknowledged,
            lossCompleteOutcomeCount = transportLoss.CompleteOutcomeCount,
            transportLoss.RemainingAmbiguousCount,
            transportLossRetainedOutcomeCount = transportLoss.CompleteOutcomes.Count,
            lossEncodedRequestBytes = transportLoss.EncodedRequestBytes,
            lossEncodedCompleteResponseBytes = transportLoss.EncodedCompleteResponseBytes,
            lossPeakAdapterRetainedOutcomeWireBytes =
                transportLoss.PeakAdapterRetainedOutcomeWireBytes,
            replacementIsExact = StringComparer.Ordinal.Equals(replacementResult, expected),
            memberAttemptsRetried = false,
            callbackShapeIsPrivateExperiment = true
        };
    }

    public static async Task<object> ExerciseCumulativeLimitAsync(string workerPath)
    {
        const int items = 5_000;
        var (stylesheet, expected) = BuildFixture(items);
        using var worker = await FastXsltWorkerClient.StartAsync(
            workerPath,
            "urn:fastxslt:incremental-limit:source",
            "<root/>"u8.ToArray(),
            "urn:fastxslt:incremental-limit:stylesheet",
            stylesheet);
        var identities = Enumerable.Range(0, 8)
            .Select(index => $"incremental-limit-{index}")
            .ToArray();
        IsolatedWorkerIncrementalBatchTerminatedException observed;
        try
        {
            _ = await worker.TransformIncrementalBatchAsync(identities);
            throw new InvalidOperationException(
                "Incremental result-pressure batch unexpectedly fit the cumulative ceiling.");
        }
        catch (IsolatedWorkerIncrementalBatchTerminatedException failure)
        {
            observed = failure;
        }

        var completePrefixIsExact = observed.CompleteOutcomes
            .Select((outcome, index) =>
                outcome.Failure is null &&
                StringComparer.Ordinal.Equals(outcome.RequestIdentity, identities[index]) &&
                StringComparer.Ordinal.Equals(outcome.Result, expected))
            .All(static exact => exact);
        var recovery = await worker.TransformAsync("incremental-limit-recovery");
        FastXsltWorkerException aggregateFailure;
        try
        {
            _ = await worker.TransformBatchAsync(identities);
            throw new InvalidOperationException(
                "Aggregate result-pressure batch unexpectedly fit the retention ceiling.");
        }
        catch (FastXsltWorkerException failure)
        {
            aggregateFailure = failure;
        }
        var aggregateRecovery = await worker.TransformAsync("aggregate-limit-recovery");
        return new
        {
            observed.Failure.Code,
            observed.Failure.Category,
            observed.Failure.Detail,
            completeOutcomeCount = observed.CompleteOutcomes.Count,
            observed.AmbiguousMemberIndex,
            observed.UnstartedMemberCount,
            completePrefixIsExact,
            ambiguousRequestIdentity = identities[observed.AmbiguousMemberIndex],
            unstartedRequestIdentities = identities
                .Skip(observed.AmbiguousMemberIndex + 1)
                .ToArray(),
            recoveryIsExact = StringComparer.Ordinal.Equals(recovery, expected),
            aggregateFailureCode = aggregateFailure.Code,
            aggregateFailureCategory = aggregateFailure.Category,
            aggregateRecoveryIsExact = StringComparer.Ordinal.Equals(
                aggregateRecovery,
                expected),
            aggregateRetentionRejectedBeforeSuffix = true,
            memberAttemptsRetried = false,
            oneSequentialExecutionLane = true
        };
    }

    public static async Task<object> ExerciseSlowConsumerAsync(string workerPath)
    {
        const int items = 5_000;
        var (stylesheet, expected) = BuildFixture(items);
        using var worker = await FastXsltWorkerClient.StartAsync(
            workerPath,
            "urn:fastxslt:incremental-backpressure:source",
            "<root/>"u8.ToArray(),
            "urn:fastxslt:incremental-backpressure:stylesheet",
            stylesheet);
        var identities = Enumerable.Range(0, 4)
            .Select(index => $"incremental-backpressure-{index}")
            .ToArray();
        var before = worker.ObserveProcess();
        var pending = worker.TransformIncrementalBatchWithReadDelayAsync(
            identities,
            TimeSpan.FromMilliseconds(30));
        var peakWorkingSetBytes = before.WorkingSetBytes;
        while (!pending.IsCompleted)
        {
            await Task.Delay(5);
            peakWorkingSetBytes = Math.Max(
                peakWorkingSetBytes,
                worker.ObserveProcess().WorkingSetBytes);
        }
        var result = await pending;
        var after = worker.ObserveProcess();
        var exact = result.Outcomes.Select((outcome, index) =>
            outcome.Failure is null &&
            StringComparer.Ordinal.Equals(outcome.RequestIdentity, identities[index]) &&
            StringComparer.Ordinal.Equals(outcome.Result, expected)).All(static value => value);
        var recovery = await worker.TransformAsync("incremental-backpressure-recovery");
        return new
        {
            memberCount = identities.Length,
            exact,
            firstOutcomeMilliseconds = result.FirstOutcomeElapsed.TotalMilliseconds,
            finalOutcomeMilliseconds = result.FinalOutcomeElapsed.TotalMilliseconds,
            imposedReadDelayMilliseconds = 30,
            minimumCumulativeDelayMilliseconds = 90,
            observedWorkingSetBeforeBytes = before.WorkingSetBytes,
            peakWorkingSetBytes,
            observedWorkingSetAfterBytes = after.WorkingSetBytes,
            recoveryIsExact = StringComparer.Ordinal.Equals(recovery, expected),
            oneSequentialExecutionLane = true,
            completedOutcomeVectorExistsOnlyInHost = true
        };
    }

    private static (byte[] Stylesheet, string Expected) BuildFixture(int items)
    {
        var stylesheet = Encoding.UTF8.GetBytes(
            $"<xsl:stylesheet version=\"3.0\" xmlns:xsl=\"http://www.w3.org/1999/XSL/Transform\">" +
            "<xsl:output method=\"xml\" omit-xml-declaration=\"yes\"/>" +
            $"<xsl:template match=\"/\"><out><xsl:for-each select=\"1 to {items}\">" +
            "<item code=\"fixed\">payload</item></xsl:for-each></out></xsl:template>" +
            "</xsl:stylesheet>");
        var expected = new StringBuilder("<out>");
        for (var index = 0; index < items; index++)
        {
            expected.Append("<item code=\"fixed\">payload</item>");
        }
        expected.Append("</out>");
        return (stylesheet, expected.ToString());
    }

    private sealed class ConsumerAbandonedIncrementalBatchException : Exception;
}
