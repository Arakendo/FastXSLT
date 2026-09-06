[CmdletBinding()]
param(
    [int]$Port = 5087,
    [ValidateSet('net8.0', 'net10.0')]
    [string]$TargetFramework = 'net10.0',
    [int]$MeasurementRequests = 1000,
    [int]$MeasurementRuns = 3,
    [switch]$LocalSaxonCs,
    [switch]$TieredBenchmark,
    [switch]$TieredSummaryOnly,
    [switch]$TieredOnly,
    [switch]$BestPracticeDeploymentBenchmark,
    [switch]$TextHeavyBenchmark,
    [switch]$ResultHeavyBenchmark,
    [switch]$NativeBoundaryBreakdown,
    [switch]$IsolatedBoundaryBreakdown,
    [switch]$IsolatedBatchBenchmark,
    [switch]$IsolatedBatchWorkerSweep,
    [switch]$IsolatedBatchResultPressure,
    [switch]$OperationalExperiments,
    [switch]$NativeRegistryPressure,
    [switch]$RegistrySummaryOnly,
    [switch]$NativeRegistryBursts,
    [switch]$NativeRegistryReplacementSoak,
    [int]$TieredRequests = 250,
    [int]$TieredConcurrency = 4,
    [int]$TieredOrderSeedBase = 17000,
    [ValidateRange(128, 20000)]
    [int]$BestPracticeMembers = 4000,
    [int]$BatchSweepMembers = 1024,
    [int]$BatchResultMembers = 256,
    [int]$TextHeavyRequests = 100,
    [int]$ResultHeavyRequests = 50,
    [int]$RegistryItems = 500,
    [int]$RegistryConcurrency = 4,
    [int]$RegistryGenerations = 2,
    [int]$RegistryDelayedOutcomes = 64,
    [ValidateRange(1000, 60000)]
    [int]$RegistrySettlementMilliseconds = 1000,
    [int]$BurstConcurrency = 8,
    [int]$BurstDelayedFailures = 128,
    [int]$BurstLargeOutcomes = 8,
    [int]$BurstLargePayloadBytes = 900000,
    [int]$SoakConcurrency = 8,
    [int]$SoakReplacements = 32,
    [int]$SoakRetainedOldGenerations = 2,
    [int]$SoakRequestsPerGeneration = 16
)

$ErrorActionPreference = 'Stop'
if ($MeasurementRuns -lt 1) {
    throw 'MeasurementRuns must be at least 1.'
}
function Get-Median([double[]]$Values) {
    $ordered = @($Values | Sort-Object)
    if ($ordered.Count % 2 -eq 1) {
        return $ordered[[int][Math]::Floor($ordered.Count / 2)]
    }
    $upper = [int]($ordered.Count / 2)
    return ($ordered[$upper - 1] + $ordered[$upper]) / 2
}
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$project = Join-Path $repositoryRoot 'workbenches/FastXSLT.AspNet.Workbench/FastXSLT.AspNet.Workbench.csproj'
$targetFramework = $TargetFramework
$workbenchDirectory = Join-Path $repositoryRoot '.workbench'
$stdoutLog = Join-Path $workbenchDirectory 'aspnet-stdout.log'
$stderrLog = Join-Path $workbenchDirectory 'aspnet-stderr.log'
$baseAddress = "http://127.0.0.1:$Port"

New-Item -ItemType Directory -Path $workbenchDirectory -Force | Out-Null

Push-Location $repositoryRoot
try {
    cargo build --release -p fastxslt-worker -p fastxslt-dotnet-workbench
    if ($LASTEXITCODE -ne 0) {
        throw "Rust worker build failed with exit code $LASTEXITCODE"
    }
    $dotnetBuildArguments = @(
        'build', $project, '--configuration', 'Release',
        "-p:FastXsltDiagnosticTargetFramework=$targetFramework")
    if ($LocalSaxonCs) {
        $dotnetBuildArguments += '-p:EnableLocalSaxonCs=true'
    }
    dotnet @dotnetBuildArguments
    if ($LASTEXITCODE -ne 0) {
        throw "ASP.NET workbench build failed with exit code $LASTEXITCODE"
    }
    $nativeLibraryName = if ($IsWindows) { 'fastxslt_dotnet_workbench.dll' } elseif ($IsMacOS) { 'libfastxslt_dotnet_workbench.dylib' } else { 'libfastxslt_dotnet_workbench.so' }
    $nativeLibrary = Join-Path $repositoryRoot "target/release/$nativeLibraryName"
    $managedOutput = Join-Path $repositoryRoot "workbenches/FastXSLT.AspNet.Workbench/bin/Release/$targetFramework"
    Copy-Item -LiteralPath $nativeLibrary -Destination $managedOutput -Force
    $managedAssembly = Join-Path $managedOutput 'FastXSLT.AspNet.Workbench.dll'
    $quotaSmoke = dotnet $managedAssembly --native-quota-smoke
    if ($LASTEXITCODE -ne 0 -or
        $quotaSmoke -cne 'native-quota-smoke: FXFFI0103 resource-exhausted') {
        throw "Native quota smoke failed: $quotaSmoke"
    }

    $server = Start-Process -FilePath 'dotnet' `
        -ArgumentList @('run', '--no-build', '--configuration', 'Release', '--project', $project, '--urls', $baseAddress) `
        -WorkingDirectory $repositoryRoot `
        -WindowStyle Hidden `
        -RedirectStandardOutput $stdoutLog `
        -RedirectStandardError $stderrLog `
        -PassThru
    try {
        $ready = $false
        for ($attempt = 0; $attempt -lt 50; $attempt++) {
            try {
                $health = Invoke-RestMethod -Uri "$baseAddress/health"
                $ready = $health.status -eq 'ready'
                if ($ready) {
                    break
                }
            }
            catch {
                Start-Sleep -Milliseconds 100
            }
        }
        if (-not $ready) {
            throw 'ASP.NET workbench did not become ready.'
        }
        if ($LocalSaxonCs -and -not $health.saxonCsAvailable) {
            throw 'The local SaxonCS overlay was requested but was not available.'
        }
        if ($LocalSaxonCs -and
            (-not $health.saxonDestinationParity.utf8BenchmarkEligible -or
             -not $health.saxonDestinationParity.failure.bothFailed)) {
            throw 'The Saxon TextWriter destination did not preserve the UTF-8 benchmark/failure oracle.'
        }
        if ($LocalSaxonCs) {
            [pscustomobject]@{
                Kind = 'SaxonDestinationParity'
                Utf8Equivalent = $health.saxonDestinationParity.utf8.equivalent
                AsciiEquivalent = $health.saxonDestinationParity.ascii.equivalent
                BothFailurePathsFailed = $health.saxonDestinationParity.failure.bothFailed
                Utf8BenchmarkEligible = $health.saxonDestinationParity.utf8BenchmarkEligible
                GeneralSerializationEligible =
                    $health.saxonDestinationParity.generalSerializationEligible
                Utf8Stream = $health.saxonDestinationParity.utf8.stream
                Utf8TextWriter = $health.saxonDestinationParity.utf8.textWriter
                AsciiStream = $health.saxonDestinationParity.ascii.stream
                AsciiTextWriter = $health.saxonDestinationParity.ascii.textWriter
            }
        }

        $result = Invoke-WebRequest -Method Post -Uri "$baseAddress/transform/smoke-001"
        $expected = '<?xml version="1.0" encoding="UTF-8"?><out>36.02</out>'
        if ($result.StatusCode -ne 200 -or $result.Content -cne $expected) {
            throw "Unexpected transform response: $($result.StatusCode) $($result.Content)"
        }
        if ($health.dotNetXslt1ExactStylesheetExecuted) {
            throw 'XslCompiledTransform unexpectedly executed the exact XSLT 2.0 stylesheet.'
        }
        if (-not $health.nativeInProcessAvailable) {
            throw 'The in-process native FastXSLT workbench was not available.'
        }
        $nativeResult = Invoke-WebRequest -Method Post -Uri "$baseAddress/transform/inprocess/native-smoke-001"
        if ($nativeResult.StatusCode -ne 200 -or $nativeResult.Content -cne $expected) {
            throw "Unexpected in-process native response: $($nativeResult.StatusCode) $($nativeResult.Content)"
        }
        $dotNetResult = Invoke-WebRequest -Method Post -Uri "$baseAddress/transform/dotnet-xslt1"
        $dotNetExpected = '<?xml version="1.0" encoding="utf-8"?><out>36.02</out>'
        if ($dotNetResult.StatusCode -ne 200 -or $dotNetResult.Content -cne $dotNetExpected) {
            throw "Unexpected .NET XSLT 1.0 response: $($dotNetResult.StatusCode) $($dotNetResult.Content)"
        }
        if ($health.saxonCsAvailable) {
            $saxonResult = Invoke-WebRequest -Method Post -Uri "$baseAddress/transform/saxoncs"
            if ($saxonResult.StatusCode -ne 200 -or $saxonResult.Content -cne $expected) {
                throw "Unexpected SaxonCS response: $($saxonResult.StatusCode) $($saxonResult.Content)"
            }
        }
        if ($OperationalExperiments) {
            $batchControls = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/isolated-batch-controls"
            if ($batchControls.outcomes.Count -ne 4 -or
                $batchControls.outcomes[0].requestIdentity -cne 'batch-control-before' -or
                $batchControls.outcomes[0].result -cne $expected -or
                $batchControls.outcomes[1].requestIdentity -cne 'batch-control-cancelled' -or
                $batchControls.outcomes[1].failureCode -cne 'FXCT0001' -or
                $batchControls.outcomes[1].failureCategory -cne 'cancelled' -or
                $batchControls.outcomes[2].requestIdentity -cne 'batch-control-limited' -or
                $batchControls.outcomes[2].failureCode -cne 'FXCT0002' -or
                $batchControls.outcomes[2].failureCategory -cne 'limit' -or
                $batchControls.outcomes[3].requestIdentity -cne 'batch-control-after' -or
                $batchControls.outcomes[3].result -cne $expected -or
                $batchControls.incrementalOutcomes.Count -ne 4 -or
                $batchControls.incrementalOutcomes[0].requestIdentity -cne 'batch-incremental-before' -or
                $batchControls.incrementalOutcomes[0].result -cne $expected -or
                $batchControls.incrementalOutcomes[1].requestIdentity -cne 'batch-incremental-cancelled' -or
                $batchControls.incrementalOutcomes[1].failureCode -cne 'FXCT0001' -or
                $batchControls.incrementalOutcomes[1].failureCategory -cne 'cancelled' -or
                $batchControls.incrementalOutcomes[2].requestIdentity -cne 'batch-incremental-limited' -or
                $batchControls.incrementalOutcomes[2].failureCode -cne 'FXCT0002' -or
                $batchControls.incrementalOutcomes[2].failureCategory -cne 'limit' -or
                $batchControls.incrementalOutcomes[3].requestIdentity -cne 'batch-incremental-after' -or
                $batchControls.incrementalOutcomes[3].result -cne $expected -or
                $batchControls.incrementalFirstOutcomeMicroseconds -le 0 -or
                $batchControls.incrementalFinalOutcomeMicroseconds -lt $batchControls.incrementalFirstOutcomeMicroseconds -or
                $batchControls.recovery -cne $expected -or
                -not $batchControls.memberControlsAreIndependent -or
                -not $batchControls.incrementalMemberControlsAreIndependent -or
                $batchControls.activeMidMemberCancellationIncluded) {
                throw "Isolated batch controls changed member independence: $($batchControls | ConvertTo-Json -Depth 5)"
            }
            $incrementalLimit = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/isolated-incremental-batch-limit"
            if ($incrementalLimit.code -cne 'FXWB1004' -or
                $incrementalLimit.category -cne 'limit' -or
                $incrementalLimit.completeOutcomeCount -ne 6 -or
                $incrementalLimit.ambiguousMemberIndex -ne 6 -or
                $incrementalLimit.ambiguousRequestIdentity -cne 'incremental-limit-6' -or
                $incrementalLimit.unstartedMemberCount -ne 1 -or
                $incrementalLimit.unstartedRequestIdentities.Count -ne 1 -or
                $incrementalLimit.unstartedRequestIdentities[0] -cne 'incremental-limit-7' -or
                -not $incrementalLimit.completePrefixIsExact -or
                -not $incrementalLimit.recoveryIsExact -or
                $incrementalLimit.aggregateFailureCode -cne 'FXWB1004' -or
                $incrementalLimit.aggregateFailureCategory -cne 'limit' -or
                -not $incrementalLimit.aggregateRecoveryIsExact -or
                -not $incrementalLimit.aggregateRetentionRejectedBeforeSuffix -or
                $incrementalLimit.memberAttemptsRetried -or
                -not $incrementalLimit.oneSequentialExecutionLane) {
                throw "Incremental batch cumulative-limit classification changed: $($incrementalLimit | ConvertTo-Json -Depth 6)"
            }
            $incrementalBackpressure = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/isolated-incremental-batch-backpressure"
            if ($incrementalBackpressure.memberCount -ne 4 -or
                -not $incrementalBackpressure.exact -or
                $incrementalBackpressure.firstOutcomeMilliseconds -le 0 -or
                ($incrementalBackpressure.finalOutcomeMilliseconds - $incrementalBackpressure.firstOutcomeMilliseconds) -lt 60 -or
                $incrementalBackpressure.imposedReadDelayMilliseconds -ne 30 -or
                $incrementalBackpressure.minimumCumulativeDelayMilliseconds -ne 90 -or
                $incrementalBackpressure.peakWorkingSetBytes -lt $incrementalBackpressure.observedWorkingSetBeforeBytes -or
                -not $incrementalBackpressure.recoveryIsExact -or
                -not $incrementalBackpressure.oneSequentialExecutionLane -or
                -not $incrementalBackpressure.completedOutcomeVectorExistsOnlyInHost) {
                throw "Incremental batch slow-consumer backpressure changed: $($incrementalBackpressure | ConvertTo-Json -Depth 6)"
            }
            $incrementalConsumer = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/isolated-incremental-batch-consumer"
            if ($incrementalConsumer.memberCount -ne 4 -or
                $incrementalConsumer.unsupportedProtocolCode -cne 'FXWB1005' -or
                $incrementalConsumer.unsupportedProtocolCategory -cne 'invalid' -or
                -not $incrementalConsumer.unsupportedProtocolRecoveryIsExact -or
                $incrementalConsumer.callbackCount -ne 4 -or
                -not $incrementalConsumer.callbackResultsAreExact -or
                $incrementalConsumer.streamedCompleteOutcomeCount -ne 4 -or
                $incrementalConsumer.retainedOutcomeCount -ne 0 -or
                $incrementalConsumer.encodedRequestBytes -ne $incrementalConsumer.expectedRequestBytes -or
                $incrementalConsumer.encodedResponseBytes -ne $incrementalConsumer.expectedResponseBytes -or
                $incrementalConsumer.peakAdapterRetainedOutcomeCount -ne 1 -or
                $incrementalConsumer.peakAdapterRetainedOutcomeWireBytes -ne $incrementalConsumer.expectedPeakAdapterRetainedOutcomeWireBytes -or
                -not $incrementalConsumer.retainingResultsAreExact -or
                $incrementalConsumer.retainingOutcomeCount -ne 4 -or
                $incrementalConsumer.retainingPeakAdapterRetainedOutcomeCount -ne 4 -or
                $incrementalConsumer.retainingPeakAdapterRetainedOutcomeWireBytes -ne $incrementalConsumer.expectedRetainingOutcomeWireBytes -or
                $incrementalConsumer.firstOutcomeMilliseconds -le 0 -or
                $incrementalConsumer.finalOutcomeMilliseconds -lt $incrementalConsumer.firstOutcomeMilliseconds -or
                -not $incrementalConsumer.recoveryIsExact -or
                $incrementalConsumer.abandonedCallbackCount -ne 2 -or
                -not $incrementalConsumer.abandonedWorkerRejectedReuse -or
                $incrementalConsumer.lossCallbackCount -ne 1 -or
                -not $incrementalConsumer.acknowledged -or
                $incrementalConsumer.lossCompleteOutcomeCount -ne 1 -or
                $incrementalConsumer.remainingAmbiguousCount -ne 3 -or
                $incrementalConsumer.transportLossRetainedOutcomeCount -ne 0 -or
                $incrementalConsumer.lossEncodedRequestBytes -ne $incrementalConsumer.expectedRequestBytes -or
                $incrementalConsumer.lossEncodedCompleteResponseBytes -le 0 -or
                $incrementalConsumer.lossPeakAdapterRetainedOutcomeWireBytes -ne $incrementalConsumer.expectedPeakAdapterRetainedOutcomeWireBytes -or
                -not $incrementalConsumer.replacementIsExact -or
                $incrementalConsumer.memberAttemptsRetried -or
                -not $incrementalConsumer.callbackShapeIsPrivateExperiment) {
                throw "Incremental batch consumer boundary changed: $($incrementalConsumer | ConvertTo-Json -Depth 6)"
            }
            $batchLoss = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/isolated-batch-loss"
            if ($batchLoss.trials.Count -ne 3 -or
                $batchLoss.recovery -cne $expected -or
                -not $batchLoss.aggregateResponseOracle -or
                $batchLoss.memberResultsTransferredBeforeLoss -ne 0 -or
                $batchLoss.killedMemberAttemptsRetried -or
                -not $batchLoss.finishedButUntransferredIsAmbiguous -or
                -not $batchLoss.laterMembersAreUnstarted) {
                throw "Isolated batch loss summary changed: $($batchLoss | ConvertTo-Json -Depth 8)"
            }
            foreach ($trial in $batchLoss.trials) {
                if ($trial.observations.Count -ne (($trial.parkAtIndex * 2) + 1) -or
                    $trial.members.Count -ne 5) {
                    throw "Isolated batch loss observation prefix changed: $($trial | ConvertTo-Json -Depth 8)"
                }
                foreach ($member in $trial.members) {
                    $expectedDisposition = if ($member.memberIndex -le $trial.parkAtIndex) {
                        'operationally-ambiguous'
                    }
                    else {
                        'unstarted'
                    }
                    if ($member.disposition -cne $expectedDisposition -or
                        $member.correlatedOutcomeTransferred) {
                        throw "Isolated batch loss classification changed: $($trial | ConvertTo-Json -Depth 8)"
                    }
                }
            }
            if ($batchLoss.transferLoss.truncateAtIndex -ne 2 -or
                $batchLoss.transferLoss.completeOutcomeCount -ne 2 -or
                $batchLoss.transferLoss.partialRequestIdentity -cne 'batch-transfer-loss-2' -or
                $batchLoss.transferLoss.observedResultBytes -le 0 -or
                $batchLoss.transferLoss.observedResultBytes -ge $batchLoss.transferLoss.declaredResultBytes -or
                $batchLoss.transferLoss.members.Count -ne 5) {
                throw "Isolated batch transfer-loss boundary changed: $($batchLoss.transferLoss | ConvertTo-Json -Depth 8)"
            }
            foreach ($member in $batchLoss.transferLoss.members) {
                $expectedDisposition = if ($member.memberIndex -lt 2) {
                    'complete'
                }
                elseif ($member.memberIndex -eq 2) {
                    'operationally-ambiguous'
                }
                else {
                    'unstarted'
                }
                $expectedTransferred = $member.memberIndex -lt 2
                if ($member.disposition -cne $expectedDisposition -or
                    $member.correlatedOutcomeTransferred -ne $expectedTransferred) {
                    throw "Isolated batch transfer-loss classification changed: $($batchLoss.transferLoss | ConvertTo-Json -Depth 8)"
                }
            }
            if ($batchLoss.malformedCommand.exitCode -eq 0 -or
                $batchLoss.malformedCommand.fullCommandDecoded -or
                $batchLoss.malformedCommand.memberAttemptsAdmitted -ne 0 -or
                $batchLoss.malformedCommand.disposition -cne 'unstarted' -or
                $batchLoss.malformedCommand.memberAttemptsRetried) {
                throw "Malformed batch command classification changed: $($batchLoss.malformedCommand | ConvertTo-Json -Depth 5)"
            }
            if ($batchLoss.unacknowledgedDispatch.memberCount -ne 5 -or
                $batchLoss.unacknowledgedDispatch.acknowledgementObserved -or
                $batchLoss.unacknowledgedDispatch.correlatedOutcomesObserved -ne 0 -or
                $batchLoss.unacknowledgedDispatch.disposition -cne 'operationally-ambiguous' -or
                $batchLoss.unacknowledgedDispatch.memberAttemptsRetried) {
                throw "Unacknowledged batch dispatch classification changed: $($batchLoss.unacknowledgedDispatch | ConvertTo-Json -Depth 5)"
            }
            if ($batchLoss.activeCancellation.cancelAtIndex -ne 2 -or
                $batchLoss.activeCancellation.outcomes.Count -ne 5 -or
                $batchLoss.activeCancellation.outcomes[0].result -cne $expected -or
                $batchLoss.activeCancellation.outcomes[1].result -cne $expected -or
                $batchLoss.activeCancellation.outcomes[2].failureCode -cne 'FXCT0001' -or
                $batchLoss.activeCancellation.outcomes[2].failureCategory -cne 'cancelled' -or
                $batchLoss.activeCancellation.outcomes[3].result -cne $expected -or
                $batchLoss.activeCancellation.outcomes[4].result -cne $expected -or
                $batchLoss.activeCancellation.recovery -cne $expected -or
                -not $batchLoss.activeCancellation.oneSequentialExecutionLane -or
                -not $batchLoss.activeCancellation.laterSiblingsExecuted) {
                throw "Active batch cancellation changed: $($batchLoss.activeCancellation | ConvertTo-Json -Depth 8)"
            }
            $batchCancellationRaces = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/isolated-batch-cancellation-races"
            if ($batchCancellationRaces.trials -ne 25 -or
                ($batchCancellationRaces.cancellations + $batchCancellationRaces.completions) -ne 25 -or
                $batchCancellationRaces.invalidTargetOutcomes -ne 0 -or
                $batchCancellationRaces.siblingFailures -ne 0 -or
                $batchCancellationRaces.recovery -cne '<?xml version="1.0" encoding="UTF-8"?><out>20000.00</out>' -or
                -not $batchCancellationRaces.completionWinsIfCommittedBeforeSignal -or
                $batchCancellationRaces.firstChargeBarrierUsed -or
                -not $batchCancellationRaces.oneSequentialExecutionLane) {
                throw "Natural batch cancellation races changed: $($batchCancellationRaces | ConvertTo-Json -Depth 6)"
            }
            $cancellation = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/cooperative-cancellation"
            if ($cancellation.cancellation.failureCode -ne 'FXCT0001' -or
                $cancellation.cancellation.failureCategory -ne 'cancelled' -or
                $cancellation.cancellation.cancelledRequestIdentity -ne 'cooperative-cancelled' -or
                $cancellation.cancellation.failureDetail -cne 'host cancellation observed while charging xslt-instruction work' -or
                $cancellation.cancellation.processIdBefore -ne $cancellation.cancellation.processIdAfter -or
                $cancellation.cancellation.recoveryResult -cne $expected -or
                -not $cancellation.cancellationWasCooperative -or
                $cancellation.workerWasTerminated -or
                $cancellation.activeMidExecutionSignalSupported) {
                throw "Cooperative cancellation experiment violated its guarantee class: $($cancellation | ConvertTo-Json -Depth 5)"
            }
            $activeCancellation = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/active-cancellation"
            if ($activeCancellation.cancellation.failureCode -ne 'FXCT0001' -or
                $activeCancellation.cancellation.failureCategory -ne 'cancelled' -or
                $activeCancellation.cancellation.cancelledRequestIdentity -ne 'active-cooperative-cancelled' -or
                -not $activeCancellation.cancellation.failureDetail.StartsWith('host cancellation observed while charging ') -or
                $activeCancellation.cancellation.processIdBefore -ne $activeCancellation.cancellation.processIdAfter -or
                -not $activeCancellation.cancellation.unrelatedSignalIgnored -or
                $activeCancellation.cancellation.recoveryResult -cne '<?xml version="1.0" encoding="UTF-8"?><out>500.00</out>' -or
                -not $activeCancellation.signalSentAfterWorkerStarted -or
                -not $activeCancellation.cancellationWasCooperative -or
                $activeCancellation.workerWasTerminated -or
                -not $activeCancellation.completionWinsIfCommittedBeforeSignal -or
                -not $activeCancellation.firstChargeBarrierWasExperimental) {
                throw "Active cancellation experiment violated its race or reuse contract: $($activeCancellation | ConvertTo-Json -Depth 5)"
            }
            $naturalCancellation = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/natural-cancellation-races"
            if ($naturalCancellation.races.trials -ne 25 -or
                ($naturalCancellation.races.cancellations + $naturalCancellation.races.completions) -ne 25 -or
                $naturalCancellation.races.cancellations -lt 1 -or
                $naturalCancellation.races.processIdBefore -ne $naturalCancellation.races.processIdAfter -or
                $naturalCancellation.races.recoveryResult -cne '<?xml version="1.0" encoding="UTF-8"?><out>20000.00</out>' -or
                $naturalCancellation.firstChargeBarrierUsed -or
                -not $naturalCancellation.completionWinsIfCommittedBeforeSignal -or
                $naturalCancellation.workerWasTerminated) {
                throw "Natural cancellation race experiment violated its accounting or reuse contract: $($naturalCancellation | ConvertTo-Json -Depth 5)"
            }
            $managedCancellation = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/managed-cancellation"
            if ($managedCancellation.cancellation.preDispatchFailureCode -ne 'FXCT0001' -or
                $managedCancellation.cancellation.preDispatchFailureCategory -ne 'cancelled' -or
                $managedCancellation.cancellation.preDispatchRequestIdentity -ne 'managed-cancellation-pre-dispatch' -or
                $managedCancellation.cancellation.preDispatchFailureDetail -cne 'host cancellation observed while charging xslt-instruction work' -or
                $managedCancellation.activeOutcome -notin @('cancelled', 'completed') -or
                ($managedCancellation.activeOutcome -eq 'cancelled' -and
                    ($managedCancellation.cancellation.activeFailureCode -ne 'FXCT0001' -or
                     $managedCancellation.cancellation.activeFailureCategory -ne 'cancelled' -or
                     $managedCancellation.cancellation.activeRequestIdentity -ne 'managed-cancellation-active')) -or
                $managedCancellation.cancellation.recoveryResult -cne '<?xml version="1.0" encoding="UTF-8"?><out>20000.00</out>' -or
                -not $managedCancellation.managedTokenMeansCooperativeRequest -or
                $managedCancellation.hardTerminationGuaranteed -or
                -not $managedCancellation.completionWinsIfCommittedBeforeSignal) {
                throw "Managed cancellation experiment violated its adapter contract: $($managedCancellation | ConvertTo-Json -Depth 5)"
            }
            $diagnostics = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/diagnostic-parity"
            if ($diagnostics.invalidIdentity.code -ne 'FXWB0003' -or
                $diagnostics.invalidIdentity.category -ne 'invalid' -or
                $null -ne $diagnostics.invalidIdentity.requestId -or
                $diagnostics.invalidIdentity.detail -cne 'request identity must not be empty' -or
                $diagnostics.malformedSource.code -ne 'FXXM0002' -or
                $diagnostics.malformedSource.category -ne 'invalid' -or
                -not $diagnostics.malformedSource.detail.Contains('urn:fastxslt:diagnostic:malformed-source') -or
                $diagnostics.unsupportedStylesheet.code -ne 'FXST1006' -or
                $diagnostics.unsupportedStylesheet.category -ne 'unsupported' -or
                $diagnostics.unsupportedStylesheet.detail -cne 'unsupported XSLT instruction: xsl:message at urn:fastxslt:diagnostic:unsupported-stylesheet:103..117' -or
                $diagnostics.cancellation.code -ne 'FXCT0001' -or
                $diagnostics.cancellation.category -ne 'cancelled' -or
                $diagnostics.cancellation.requestId -ne 'diagnostic-cancelled' -or
                $diagnostics.processIdBefore -ne $diagnostics.processIdAfter -or
                $diagnostics.recoveryResult -cne $expected -or
                -not $diagnostics.sameDiagnosticFieldsAsDirectRustAssertions) {
                throw "Diagnostic parity experiment changed a direct-path diagnostic: $($diagnostics | ConvertTo-Json -Depth 5)"
            }
            $instructionBudget = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/instruction-budget"
            if ($instructionBudget.exhaustion.code -ne 'FXCT0002' -or
                $instructionBudget.exhaustion.category -ne 'limit' -or
                $instructionBudget.exhaustion.requestId -ne 'instruction-budget-exhausted' -or
                $instructionBudget.exhaustion.detail -cne 'xslt-instruction work budget exhausted: limit 0, consumed 0, next charge 1' -or
                $instructionBudget.configuredMaximumXsltInstructions -ne 0 -or
                $instructionBudget.processIdBefore -ne $instructionBudget.processIdAfter -or
                $instructionBudget.recoveryResult -cne $expected -or
                -not $instructionBudget.deterministicEngineBudget -or
                $instructionBudget.cooperativeCancellation -or
                $instructionBudget.workerWasTerminated -or
                $instructionBudget.requestWasRetried) {
                throw "Instruction budget experiment violated its guarantee class: $($instructionBudget | ConvertTo-Json -Depth 5)"
            }
            $nativeBoundary = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/native-boundary"
            if ($nativeBoundary.invalidIdentity.code -ne 'FXWB0003' -or
                $nativeBoundary.invalidIdentity.category -ne 'invalid' -or
                $nativeBoundary.invalidIdentity.detail -cne 'request identity must not be empty' -or
                $nativeBoundary.malformedSource.code -ne 'FXXM0002' -or
                $nativeBoundary.malformedSource.category -ne 'invalid' -or
                -not $nativeBoundary.malformedSource.detail.Contains('urn:fastxslt:native-boundary:malformed-source') -or
                $nativeBoundary.unsupportedStylesheet.code -ne 'FXST1006' -or
                $nativeBoundary.unsupportedStylesheet.category -ne 'unsupported' -or
                $nativeBoundary.unsupportedStylesheet.detail -cne 'unsupported XSLT instruction: xsl:message at urn:fastxslt:diagnostic:unsupported-stylesheet:103..117' -or
                $nativeBoundary.cancellation.code -ne 'FXCT0001' -or
                $nativeBoundary.cancellation.category -ne 'cancelled' -or
                $nativeBoundary.cancellation.requestId -ne 'native-controlled-cancelled' -or
                $nativeBoundary.cancellation.detail -cne 'host cancellation observed while charging xslt-instruction work' -or
                $nativeBoundary.instructionBudget.code -ne 'FXCT0002' -or
                $nativeBoundary.instructionBudget.category -ne 'limit' -or
                $nativeBoundary.instructionBudget.requestId -ne 'native-instruction-budget' -or
                $nativeBoundary.instructionBudget.detail -cne 'xslt-instruction work budget exhausted: limit 0, consumed 0, next charge 1' -or
                $nativeBoundary.recoveryResult -cne $expected -or
                $nativeBoundary.controlledRecoveryResult -cne $expected -or
                $nativeBoundary.concurrentResults.Count -ne 2 -or
                $nativeBoundary.concurrentResults[0] -cne $expected -or
                $nativeBoundary.concurrentResults[1] -cne $expected -or
                -not $nativeBoundary.independentHandlesExecutedConcurrently -or
                -not $nativeBoundary.controlsWereScalarAndPreDispatch -or
                $nativeBoundary.activeMidExecutionSignalSupported -or
                $nativeBoundary.hardTerminationGuaranteed -or
                -not $nativeBoundary.doubleDisposeWasIdempotent -or
                -not $nativeBoundary.useAfterDisposeRejected) {
                throw "Native boundary experiment violated ABI ownership or parity: $($nativeBoundary | ConvertTo-Json -Depth 5)"
            }
            $nativeReplacement = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/native-generation-replacement"
            $nativeOldExpected = '<?xml version="1.0" encoding="UTF-8"?><out>1.00</out>'
            $nativeNewExpected = '<?xml version="1.0" encoding="UTF-8"?><out>2.00</out>'
            if ($nativeReplacement.retiredGenerationIdentity -ne 'native-generation-001' -or
                $nativeReplacement.oldLeaseGenerationIdentity -ne 'native-generation-001' -or
                $nativeReplacement.newGeneration.generationIdentity -ne 'native-generation-002' -or
                $nativeReplacement.oldResult -cne $nativeOldExpected -or
                $nativeReplacement.newGeneration.result -cne $nativeNewExpected -or
                -not $nativeReplacement.replacementInitializedBeforePromotion -or
                -not $nativeReplacement.promotionWasExplicit -or
                -not $nativeReplacement.oldGenerationDisposedAfterLeaseRelease) {
                throw "Native generation replacement violated its expected lifecycle: $($nativeReplacement | ConvertTo-Json -Depth 5)"
            }
            $nativeActiveCancellation = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/native-active-cancellation"
            if ($nativeActiveCancellation.cancellation.code -ne 'FXCT0001' -or
                $nativeActiveCancellation.cancellation.category -ne 'cancelled' -or
                $nativeActiveCancellation.cancellation.requestId -ne 'native-active-cancelled' -or
                $nativeActiveCancellation.cancellation.detail -cne 'host cancellation observed while charging xslt-instruction work' -or
                $nativeActiveCancellation.signalToObservationMilliseconds -lt 0 -or
                -not $nativeActiveCancellation.firstChargeObserved -or
                -not $nativeActiveCancellation.unrelatedSignalIgnored -or
                -not $nativeActiveCancellation.controlDoubleDisposeWasIdempotent -or
                $nativeActiveCancellation.recoveryResult -cne '<?xml version="1.0" encoding="UTF-8"?><out>20000.00</out>' -or
                -not $nativeActiveCancellation.cancellationWasCooperative -or
                -not $nativeActiveCancellation.completionWinsIfCommittedBeforeSignal -or
                $nativeActiveCancellation.hardTerminationGuaranteed -or
                -not $nativeActiveCancellation.firstChargeBarrierWasExperimental) {
                throw "Native active cancellation violated its guarantee class: $($nativeActiveCancellation | ConvertTo-Json -Depth 5)"
            }
            $nativeNaturalCancellation = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/native-natural-cancellation-races"
            if ($nativeNaturalCancellation.trials -ne 25 -or
                ($nativeNaturalCancellation.cancellations + $nativeNaturalCancellation.completions) -ne 25 -or
                $nativeNaturalCancellation.cancellations -lt 1 -or
                $nativeNaturalCancellation.minimumCancellationMilliseconds -lt 0 -or
                $nativeNaturalCancellation.medianCancellationMilliseconds -lt 0 -or
                $nativeNaturalCancellation.maximumCancellationMilliseconds -lt 0 -or
                $nativeNaturalCancellation.observedChargeDetails.Count -lt 1 -or
                $nativeNaturalCancellation.recoveryResult -cne '<?xml version="1.0" encoding="UTF-8"?><out>20000.00</out>' -or
                $nativeNaturalCancellation.firstChargeBarrierUsed -or
                -not $nativeNaturalCancellation.managedCancellationTokenAdapted -or
                -not $nativeNaturalCancellation.diagnosticFieldsValidated -or
                -not $nativeNaturalCancellation.completionWinsIfCommittedBeforeSignal -or
                $nativeNaturalCancellation.hardTerminationGuaranteed) {
                throw "Native natural cancellation races violated conservation or reuse: $($nativeNaturalCancellation | ConvertTo-Json -Depth 5)"
            }
            $controlFrames = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/worker-control-frame-serialization"
            if ($controlFrames.pairs -ne 10000 -or
                $controlFrames.framesExpected -ne 20000 -or
                $controlFrames.framesObserved -ne 20000 -or
                -not $controlFrames.framesIntact -or
                -not $controlFrames.writesWereFragmentedAfterEveryByte -or
                -not $controlFrames.outboundControlFramesSerialized) {
                throw "Worker control-frame serialization stress failed: $($controlFrames | ConvertTo-Json -Depth 5)"
            }
            $recovery = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/worker-recovery"
            if ($recovery.recovery.failureCode -ne 'FXWB2001' -or
                $recovery.recovery.failureCategory -ne 'worker-terminated' -or
                $recovery.recovery.failedRequestIdentity -ne 'recovery-failed' -or
                $recovery.recovery.formerProcessId -eq $recovery.recovery.replacementProcessId -or
                $recovery.failedRequestRetried -or
                $recovery.siblingResult -cne $expected -or
                $recovery.recovery.recoveryResult -cne $expected) {
                throw "Worker recovery experiment violated its expected disposition: $($recovery | ConvertTo-Json -Depth 5)"
            }
            $replacement = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/generation-replacement"
            if ($replacement.retiredGenerationIdentity -ne 'generation-001' -or
                $replacement.oldLeaseGenerationIdentity -ne 'generation-001' -or
                $replacement.newGeneration.generationIdentity -ne 'generation-002' -or
                $replacement.oldResult -cne $expected -or
                $replacement.newGeneration.result -cne $expected -or
                -not $replacement.promotionWasExplicit -or
                -not $replacement.oldGenerationDrainsOnLeaseRelease) {
                throw "Generation replacement experiment violated its expected lifecycle: $($replacement | ConvertTo-Json -Depth 5)"
            }
            $fileReplacement = Invoke-RestMethod -Method Post -Uri "$baseAddress/experiment/host-file-replacement"
            $oldFileExpected = '<?xml version="1.0" encoding="UTF-8"?><out>1.00</out>'
            $newFileExpected = '<?xml version="1.0" encoding="UTF-8"?><out>2.00</out>'
            if ($fileReplacement.retiredGeneration -ne 'file-generation-001' -or
                $fileReplacement.oldGenerationIdentity -ne 'file-generation-001' -or
                $fileReplacement.newGeneration.generationIdentity -ne 'file-generation-002' -or
                $fileReplacement.oldResult -cne $oldFileExpected -or
                $fileReplacement.newGeneration.result -cne $newFileExpected -or
                -not $fileReplacement.importedHandlesClosedBeforePromotion -or
                -not $fileReplacement.originalFilesRenamedAndRemovedWhileGenerationWasLive -or
                -not $fileReplacement.sourceBytesChanged) {
                throw "Host file replacement violated snapshot isolation: $($fileReplacement | ConvertTo-Json -Depth 5)"
            }
            [pscustomobject]@{
                Experiment = 'BatchNaturalCancellationRaces'
                Trials = $batchCancellationRaces.trials
                Cancellations = $batchCancellationRaces.cancellations
                Completions = $batchCancellationRaces.completions
                InvalidTargetOutcomes = $batchCancellationRaces.invalidTargetOutcomes
                SiblingFailures = $batchCancellationRaces.siblingFailures
                WorkerReused = $batchCancellationRaces.recovery -ceq '<?xml version="1.0" encoding="UTF-8"?><out>20000.00</out>'
            }
            [pscustomobject]@{
                Experiment = 'IncrementalBatchCumulativeLimit'
                FailureCode = $incrementalLimit.code
                CompletePrefix = $incrementalLimit.completeOutcomeCount
                AmbiguousIndex = $incrementalLimit.ambiguousMemberIndex
                UnstartedSuffix = $incrementalLimit.unstartedMemberCount
                WorkerReused = $incrementalLimit.recoveryIsExact
                AggregateFailureCode = $incrementalLimit.aggregateFailureCode
                AggregateWorkerReused = $incrementalLimit.aggregateRecoveryIsExact
            }
            [pscustomobject]@{
                Experiment = 'IncrementalBatchBackpressure'
                Members = $incrementalBackpressure.memberCount
                FirstOutcomeMilliseconds = $incrementalBackpressure.firstOutcomeMilliseconds
                FinalOutcomeMilliseconds = $incrementalBackpressure.finalOutcomeMilliseconds
                WorkingSetBeforeBytes = $incrementalBackpressure.observedWorkingSetBeforeBytes
                PeakWorkingSetBytes = $incrementalBackpressure.peakWorkingSetBytes
                WorkingSetAfterBytes = $incrementalBackpressure.observedWorkingSetAfterBytes
                WorkerReused = $incrementalBackpressure.recoveryIsExact
            }
            [pscustomobject]@{
                Experiment = 'IncrementalBatchConsumerBoundary'
                Members = $incrementalConsumer.memberCount
                DeliveredCallbacks = $incrementalConsumer.callbackCount
                RetainedOutcomes = $incrementalConsumer.retainedOutcomeCount
                RequestWireBytes = $incrementalConsumer.encodedRequestBytes
                ResponseWireBytes = $incrementalConsumer.encodedResponseBytes
                PeakRetainedOutcomeWireBytes = $incrementalConsumer.peakAdapterRetainedOutcomeWireBytes
                CollectingPeakOutcomeCount = $incrementalConsumer.retainingPeakAdapterRetainedOutcomeCount
                CollectingPeakOutcomeWireBytes = $incrementalConsumer.retainingPeakAdapterRetainedOutcomeWireBytes
                FirstOutcomeMilliseconds = $incrementalConsumer.firstOutcomeMilliseconds
                FinalOutcomeMilliseconds = $incrementalConsumer.finalOutcomeMilliseconds
                AbandonedPrefix = $incrementalConsumer.abandonedCallbackCount
                AbandonedWorkerRetired = $incrementalConsumer.abandonedWorkerRejectedReuse
                LossObservedPrefix = $incrementalConsumer.lossCompleteOutcomeCount
                LossAmbiguousRemainder = $incrementalConsumer.remainingAmbiguousCount
                ReplacementRecovered = $incrementalConsumer.replacementIsExact
            }
            [pscustomobject]@{
                Experiment = 'CooperativeCancellation'
                FailureCode = $cancellation.cancellation.failureCode
                FailureCategory = $cancellation.cancellation.failureCategory
                WorkerReused = $cancellation.cancellation.processIdBefore -eq $cancellation.cancellation.processIdAfter
                RecoveryCompleted = $cancellation.cancellation.recoveryResult -ceq $expected
                ActiveMidExecutionSignalSupported = $cancellation.activeMidExecutionSignalSupported
            }
            [pscustomobject]@{
                Experiment = 'ActiveCancellation'
                FailureCode = $activeCancellation.cancellation.failureCode
                FailureDetail = $activeCancellation.cancellation.failureDetail
                SignalToObservationMilliseconds = $activeCancellation.cancellation.signalToObservationMilliseconds
                WorkerReused = $activeCancellation.cancellation.processIdBefore -eq $activeCancellation.cancellation.processIdAfter
                UnrelatedSignalIgnored = $activeCancellation.cancellation.unrelatedSignalIgnored
                RecoveryCompleted = $activeCancellation.cancellation.recoveryResult -ceq '<?xml version="1.0" encoding="UTF-8"?><out>500.00</out>'
            }
            [pscustomobject]@{
                Experiment = 'NaturalCancellationRaces'
                Trials = $naturalCancellation.races.trials
                Cancellations = $naturalCancellation.races.cancellations
                Completions = $naturalCancellation.races.completions
                MinimumCancellationMilliseconds = $naturalCancellation.races.minimumCancellationMilliseconds
                MedianCancellationMilliseconds = $naturalCancellation.races.medianCancellationMilliseconds
                MaximumCancellationMilliseconds = $naturalCancellation.races.maximumCancellationMilliseconds
                WorkerReused = $naturalCancellation.races.processIdBefore -eq $naturalCancellation.races.processIdAfter
            }
            [pscustomobject]@{
                Experiment = 'ManagedCancellation'
                PreDispatchFailureCode = $managedCancellation.cancellation.preDispatchFailureCode
                ActiveOutcome = $managedCancellation.activeOutcome
                ActiveFailureCode = $managedCancellation.cancellation.activeFailureCode
                RecoveryCompleted = $managedCancellation.cancellation.recoveryResult -ceq '<?xml version="1.0" encoding="UTF-8"?><out>20000.00</out>'
                HardTerminationGuaranteed = $managedCancellation.hardTerminationGuaranteed
            }
            [pscustomobject]@{
                Experiment = 'DiagnosticParity'
                InvalidIdentity = $diagnostics.invalidIdentity.code
                MalformedSource = $diagnostics.malformedSource.code
                UnsupportedStylesheet = $diagnostics.unsupportedStylesheet.code
                Cancellation = $diagnostics.cancellation.code
                WorkerReused = $diagnostics.processIdBefore -eq $diagnostics.processIdAfter
            }
            [pscustomobject]@{
                Experiment = 'InstructionBudget'
                FailureCode = $instructionBudget.exhaustion.code
                FailureCategory = $instructionBudget.exhaustion.category
                FailureDetail = $instructionBudget.exhaustion.detail
                WorkerReused = $instructionBudget.processIdBefore -eq $instructionBudget.processIdAfter
                RecoveryCompleted = $instructionBudget.recoveryResult -ceq $expected
            }
            [pscustomobject]@{
                Experiment = 'NativeBoundary'
                InvalidIdentity = $nativeBoundary.invalidIdentity.code
                MalformedSource = $nativeBoundary.malformedSource.code
                UnsupportedStylesheet = $nativeBoundary.unsupportedStylesheet.code
                PreDispatchCancellation = $nativeBoundary.cancellation.code
                InstructionBudget = $nativeBoundary.instructionBudget.code
                ControlledRecovery = $nativeBoundary.controlledRecoveryResult -ceq $expected
                ConcurrentIndependentHandles = $nativeBoundary.independentHandlesExecutedConcurrently
                DoubleDisposeIdempotent = $nativeBoundary.doubleDisposeWasIdempotent
                UseAfterDisposeRejected = $nativeBoundary.useAfterDisposeRejected
            }
            [pscustomobject]@{
                Experiment = 'NativeGenerationReplacement'
                RetiredGeneration = $nativeReplacement.retiredGenerationIdentity
                NewGeneration = $nativeReplacement.newGeneration.generationIdentity
                OldResultRetained = $nativeReplacement.oldResult -ceq $nativeOldExpected
                NewResultPromoted = $nativeReplacement.newGeneration.result -ceq $nativeNewExpected
                OldGenerationDrained = $nativeReplacement.oldGenerationDisposedAfterLeaseRelease
            }
            [pscustomobject]@{
                Experiment = 'NativeActiveCancellation'
                FailureCode = $nativeActiveCancellation.cancellation.code
                FailureDetail = $nativeActiveCancellation.cancellation.detail
                SignalToObservationMilliseconds = $nativeActiveCancellation.signalToObservationMilliseconds
                FirstChargeObserved = $nativeActiveCancellation.firstChargeObserved
                UnrelatedSignalIgnored = $nativeActiveCancellation.unrelatedSignalIgnored
                RecoveryCompleted = $nativeActiveCancellation.recoveryResult -ceq '<?xml version="1.0" encoding="UTF-8"?><out>20000.00</out>'
                HardTerminationGuaranteed = $nativeActiveCancellation.hardTerminationGuaranteed
            }
            [pscustomobject]@{
                Experiment = 'NativeNaturalCancellationRaces'
                Trials = $nativeNaturalCancellation.trials
                Cancellations = $nativeNaturalCancellation.cancellations
                Completions = $nativeNaturalCancellation.completions
                MinimumCancellationMilliseconds = $nativeNaturalCancellation.minimumCancellationMilliseconds
                MedianCancellationMilliseconds = $nativeNaturalCancellation.medianCancellationMilliseconds
                MaximumCancellationMilliseconds = $nativeNaturalCancellation.maximumCancellationMilliseconds
                ObservedChargeDetails = $nativeNaturalCancellation.observedChargeDetails -join '; '
                RecoveryCompleted = $nativeNaturalCancellation.recoveryResult -ceq '<?xml version="1.0" encoding="UTF-8"?><out>20000.00</out>'
            }
            [pscustomobject]@{
                Experiment = 'WorkerRecovery'
                FailureCode = $recovery.recovery.failureCode
                FailedRequestRetried = $recovery.failedRequestRetried
                FormerProcessId = $recovery.recovery.formerProcessId
                ReplacementProcessId = $recovery.recovery.replacementProcessId
                SiblingCompleted = $recovery.siblingResult -ceq $expected
                RecoveryCompleted = $recovery.recovery.recoveryResult -ceq $expected
            }
            [pscustomobject]@{
                Experiment = 'GenerationReplacement'
                RetiredGeneration = $replacement.retiredGenerationIdentity
                NewGeneration = $replacement.newGeneration.generationIdentity
                OldLeaseCompleted = $replacement.oldResult -ceq $expected
                NewRequestCompleted = $replacement.newGeneration.result -ceq $expected
            }
            [pscustomobject]@{
                Experiment = 'HostFileReplacement'
                RetiredGeneration = $fileReplacement.retiredGeneration
                NewGeneration = $fileReplacement.newGeneration.generationIdentity
                OldResultRetained = $fileReplacement.oldResult -ceq $oldFileExpected
                NewResultPromoted = $fileReplacement.newGeneration.result -ceq $newFileExpected
                OriginalFilesReleased = $fileReplacement.originalFilesRenamedAndRemovedWhileGenerationWasLive
            }
        }
        if ($NativeRegistryPressure) {
            $registryPressureUri = "$baseAddress/experiment/native-registry-pressure?items=$RegistryItems&concurrency=$RegistryConcurrency&generations=$RegistryGenerations&delayedOutcomes=$RegistryDelayedOutcomes&settlementMilliseconds=$RegistrySettlementMilliseconds"
            $registryPressure = Invoke-RestMethod -Method Post -Uri $registryPressureUri
            $baseline = $registryPressure.checkpoints[0].registry
            if (-not $registryPressure.logicalRegistryReturnedToBaseline -or
                $registryPressure.abandonedHandles -ne 0 -or
                $registryPressure.legitimateHighWater.engineHandles -ne ($baseline.engineHandles + ($RegistryConcurrency * $RegistryGenerations)) -or
                $registryPressure.legitimateHighWater.outcomeHandles -ne ($baseline.outcomeHandles + $RegistryDelayedOutcomes) -or
                $registryPressure.legitimateHighWater.outcomePayloadBytes -le $baseline.outcomePayloadBytes -or
                $registryPressure.settlement[-1].millisecondsAfterRelease -ne $RegistrySettlementMilliseconds -or
                $registryPressure.semanticSentinel -cne "<?xml version=`"1.0`" encoding=`"UTF-8`"?><out>$RegistryItems.00</out>") {
                throw "Native registry-pressure experiment violated lifecycle accounting or semantic parity: $($registryPressure | ConvertTo-Json -Depth 8)"
            }
            if ($RegistrySummaryOnly) {
                $allWorkingSet = @($registryPressure.checkpoints.workingSetBytes) + @($registryPressure.settlement.checkpoint.workingSetBytes)
                $allPrivateBytes = @($registryPressure.checkpoints.privateMemoryBytes) + @($registryPressure.settlement.checkpoint.privateMemoryBytes)
                [pscustomobject]@{
                    Experiment = 'NativeRegistryPressure'
                    Items = $registryPressure.items
                    Concurrency = $registryPressure.concurrency
                    Generations = $registryPressure.generations
                    DelayedOutcomes = $registryPressure.delayedOutcomes
                    EngineHighWater = $registryPressure.legitimateHighWater.engineHandles
                    OutcomeHighWater = $registryPressure.legitimateHighWater.outcomeHandles
                    OutcomePayloadHighWater = $registryPressure.legitimateHighWater.outcomePayloadBytes
                    LogicalRegistryReturnedToBaseline = $registryPressure.logicalRegistryReturnedToBaseline
                    WorkingSetBaseline = $registryPressure.checkpoints[0].workingSetBytes
                    WorkingSetPeak = ($allWorkingSet | Measure-Object -Maximum).Maximum
                    SettlementMilliseconds = $registryPressure.settlement[-1].millisecondsAfterRelease
                    WorkingSetAfterSettlement = $registryPressure.settlement[-1].checkpoint.workingSetBytes
                    PrivateBytesBaseline = $registryPressure.checkpoints[0].privateMemoryBytes
                    PrivateBytesPeak = ($allPrivateBytes | Measure-Object -Maximum).Maximum
                    PrivateBytesAfterSettlement = $registryPressure.settlement[-1].checkpoint.privateMemoryBytes
                }
                foreach ($sample in $registryPressure.settlement) {
                    [pscustomobject]@{
                        Experiment = 'NativeRegistrySettlement'
                        MillisecondsAfterRelease = $sample.millisecondsAfterRelease
                        WorkingSetBytes = $sample.checkpoint.workingSetBytes
                        PrivateBytes = $sample.checkpoint.privateMemoryBytes
                        ManagedHeapBytes = $sample.checkpoint.managedHeapBytes
                    }
                }
            }
            else {
                $registryPressure | ConvertTo-Json -Depth 8
            }
        }
        if ($NativeRegistryBursts) {
            $registryBurstUri = "$baseAddress/experiment/native-registry-bursts?concurrency=$BurstConcurrency&delayedFailures=$BurstDelayedFailures&largeOutcomes=$BurstLargeOutcomes&largePayloadBytes=$BurstLargePayloadBytes"
            $registryBurst = Invoke-RestMethod -Method Post -Uri $registryBurstUri
            $burstBaseline = $registryBurst.checkpoints[0].registry
            $expectedBurstEngineHighWater = $burstBaseline.engineHandles + $BurstConcurrency + [Math]::Min($BurstConcurrency, 8)
            if (-not $registryBurst.logicalRegistryReturnedToBaseline -or
                $registryBurst.legitimateHighWater.engineHandles -ne $expectedBurstEngineHighWater -or
                $registryBurst.legitimateHighWater.controlHandles -ne ($burstBaseline.controlHandles + $BurstConcurrency) -or
                $registryBurst.legitimateHighWater.outcomeHandles -ne ($burstBaseline.outcomeHandles + $BurstDelayedFailures + $BurstLargeOutcomes) -or
                $registryBurst.legitimateHighWater.outcomePayloadBytes -le ($BurstLargeOutcomes * $BurstLargePayloadBytes) -or
                $registryBurst.settlement.Count -ne 6) {
                throw "Native registry-burst experiment violated ownership, payload, or reclamation accounting: $($registryBurst | ConvertTo-Json -Depth 8)"
            }
            if ($RegistrySummaryOnly) {
                [pscustomobject]@{
                    Experiment = 'NativeRegistryBursts'
                    Concurrency = $registryBurst.concurrency
                    DelayedFailures = $registryBurst.delayedFailures
                    LargeOutcomes = $registryBurst.largeOutcomes
                    LargePayloadBytes = $registryBurst.largePayloadBytes
                    EngineHighWater = $registryBurst.legitimateHighWater.engineHandles
                    ControlHighWater = $registryBurst.legitimateHighWater.controlHandles
                    OutcomeHighWater = $registryBurst.legitimateHighWater.outcomeHandles
                    OutcomePayloadHighWater = $registryBurst.legitimateHighWater.outcomePayloadBytes
                    ActiveControlBurstMilliseconds = $registryBurst.activeControlBurstMilliseconds
                    FailureBurstMilliseconds = $registryBurst.failureBurstMilliseconds
                    LargeResultBurstMilliseconds = $registryBurst.largeResultBurstMilliseconds
                    LogicalRegistryReturnedToBaseline = $registryBurst.logicalRegistryReturnedToBaseline
                }
            }
            else {
                $registryBurst | ConvertTo-Json -Depth 8
            }
        }
        if ($NativeRegistryReplacementSoak) {
            $replacementSoakUri = "$baseAddress/experiment/native-registry-replacement-soak?concurrency=$SoakConcurrency&replacements=$SoakReplacements&retainedOldGenerations=$SoakRetainedOldGenerations&requestsPerGeneration=$SoakRequestsPerGeneration"
            $replacementSoak = Invoke-RestMethod -Method Post -Uri $replacementSoakUri
            $soakBaseline = $replacementSoak.checkpoints[0].registry
            $expectedEngineHighWater = $soakBaseline.engineHandles +
                ($SoakConcurrency * ([Math]::Min($SoakRetainedOldGenerations, $SoakReplacements) + 1))
            if (-not $replacementSoak.logicalRegistryReturnedToBaseline -or
                $replacementSoak.replacementSamples -ne $SoakReplacements -or
                $replacementSoak.transformSamples -ne ($SoakReplacements * $SoakRequestsPerGeneration) -or
                $replacementSoak.legitimateHighWater.engineHandles -ne $expectedEngineHighWater -or
                $replacementSoak.legitimateHighWater.controlHandles -ne $soakBaseline.controlHandles -or
                $replacementSoak.legitimateHighWater.outcomeHandles -ne $soakBaseline.outcomeHandles -or
                $replacementSoak.replacementP99Microseconds -lt $replacementSoak.replacementP50Microseconds -or
                $replacementSoak.transformP99Microseconds -lt $replacementSoak.transformP50Microseconds) {
                throw "Native registry replacement soak violated lifecycle, semantics, or accounting: $($replacementSoak | ConvertTo-Json -Depth 8)"
            }
            if ($RegistrySummaryOnly) {
                [pscustomobject]@{
                    Experiment = 'NativeRegistryReplacementSoak'
                    Concurrency = $replacementSoak.concurrency
                    Replacements = $replacementSoak.replacements
                    RetainedOldGenerations = $replacementSoak.retainedOldGenerations
                    RequestsPerGeneration = $replacementSoak.requestsPerGeneration
                    EngineHighWater = $replacementSoak.legitimateHighWater.engineHandles
                    ReplacementP50Microseconds = $replacementSoak.replacementP50Microseconds
                    ReplacementP95Microseconds = $replacementSoak.replacementP95Microseconds
                    ReplacementP99Microseconds = $replacementSoak.replacementP99Microseconds
                    TransformP50Microseconds = $replacementSoak.transformP50Microseconds
                    TransformP95Microseconds = $replacementSoak.transformP95Microseconds
                    TransformP99Microseconds = $replacementSoak.transformP99Microseconds
                    LogicalRegistryReturnedToBaseline = $replacementSoak.logicalRegistryReturnedToBaseline
                }
            }
            else {
                $replacementSoak | ConvertTo-Json -Depth 8
            }
        }
        if (-not $TieredOnly) {
            for ($run = 1; $run -le $MeasurementRuns; $run++) {
                $fastXslt = Invoke-RestMethod -Method Post -Uri "$baseAddress/measure?requests=$MeasurementRequests"
                $nativeFastXslt = Invoke-RestMethod -Method Post -Uri "$baseAddress/measure/inprocess?requests=$MeasurementRequests"
                $dotNetXslt1 = Invoke-RestMethod -Method Post -Uri "$baseAddress/measure/dotnet-xslt1?requests=$MeasurementRequests"
                $saxonCs = if ($health.saxonCsAvailable) {
                    Invoke-RestMethod -Method Post -Uri "$baseAddress/measure/saxoncs?requests=$MeasurementRequests"
                }
                else {
                    $null
                }
                [pscustomobject]@{
                    Run = $run
                    Mode = $health.mode
                    MaximumInFlight = $health.maximumInFlight
                    Requests = $fastXslt.requests
                    FastXsltElapsedMilliseconds = $fastXslt.elapsedMilliseconds
                    FastXsltTransformsPerSecond = $fastXslt.transformsPerSecond
                    NativeFastXsltElapsedMilliseconds = $nativeFastXslt.elapsedMilliseconds
                    NativeFastXsltTransformsPerSecond = $nativeFastXslt.transformsPerSecond
                    IsolatedToNativeRatio = $nativeFastXslt.transformsPerSecond / $fastXslt.transformsPerSecond
                    DotNetXslt1ElapsedMilliseconds = $dotNetXslt1.elapsedMilliseconds
                    DotNetXslt1TransformsPerSecond = $dotNetXslt1.transformsPerSecond
                    DotNetToFastXsltRatio = $dotNetXslt1.transformsPerSecond / $fastXslt.transformsPerSecond
                    SaxonCsElapsedMilliseconds = if ($saxonCs) { $saxonCs.elapsedMilliseconds } else { $null }
                    SaxonCsTransformsPerSecond = if ($saxonCs) { $saxonCs.transformsPerSecond } else { $null }
                    SaxonCsToFastXsltRatio = if ($saxonCs) { $saxonCs.transformsPerSecond / $fastXslt.transformsPerSecond } else { $null }
                    ExactStylesheetExecutedByDotNet = $health.dotNetXslt1ExactStylesheetExecuted
                    ExactStylesheetDotNetDiagnostic = $health.dotNetXslt1ExactStylesheetDiagnostic
                }
            }
        }
        if ($TieredBenchmark) {
            for ($tieredRun = 1; $tieredRun -le $MeasurementRuns; $tieredRun++) {
                $orderSeed = $TieredOrderSeedBase + $tieredRun
                $tiered = Invoke-RestMethod -Method Post -Uri "$baseAddress/benchmark/tiers?requests=$TieredRequests&concurrency=$TieredConcurrency&orderSeed=$orderSeed"
                if ($TieredSummaryOnly) {
                    $tiered.warmups | Select-Object `
                        @{Name='Run'; Expression={$tieredRun}}, `
                        @{Name='OrderSeed'; Expression={$tiered.orderSeed}}, `
                        engine, tier, totalCalls, windows, stabilized, `
                        finalRelativeMedianDrift, finalRelativeMedianAbsoluteDeviation, `
                        windowThroughputsPerSecond
                    $tiered.measurements | Select-Object `
                        @{Name='Run'; Expression={$tieredRun}}, `
                        @{Name='OrderSeed'; Expression={$tiered.orderSeed}}, `
                        measurementPosition, engine, tier, requests, concurrency, `
                        achievedConcurrencyHighWater, `
                        threadPoolThreadsBefore, threadPoolThreadsAfter, measurementProtocol, `
                        elapsedMilliseconds, transformsPerSecond, `
                        p50Microseconds, p95Microseconds, p99Microseconds, `
                        processorMilliseconds, normalizedProcessorPercent, managedAllocatedBytes, `
                        workerWorkingSetAfter
                }
                else {
                    $tiered | ConvertTo-Json -Depth 6
                }
            }
        }
        if ($BestPracticeDeploymentBenchmark) {
            for ($bestPracticeRun = 1; $bestPracticeRun -le $MeasurementRuns; $bestPracticeRun++) {
                $orderSeed = $TieredOrderSeedBase + $bestPracticeRun
                $bestPractice = Invoke-RestMethod -Method Post -Uri "$baseAddress/benchmark/best-practice-deployment?members=$BestPracticeMembers&concurrency=$TieredConcurrency&orderSeed=$orderSeed"
                if ($TieredSummaryOnly) {
                    $bestPractice.measurements | Select-Object `
                        @{Name='Run'; Expression={$bestPracticeRun}}, `
                        @{Name='OrderSeed'; Expression={$bestPractice.orderSeed}}, `
                        measurementPosition, engine, mode, tier, members, concurrency, `
                        batchSize, transactions, elapsedMilliseconds, transformsPerSecond, `
                        firstResultP50Microseconds, firstResultP95Microseconds, `
                        finalResultP50Microseconds, finalResultP95Microseconds, `
                        processorMilliseconds, managedAllocatedBytesPerMember, `
                        workerWorkingSetAfterBytes, requestWireBytes, responseWireBytes, `
                        maximumAmbiguousMembersPerWorkerLoss, `
                        maximumAggregateAmbiguousMembers, achievedConcurrencyHighWater
                }
                else {
                    $bestPractice | ConvertTo-Json -Depth 6
                }
            }
        }
        if ($TextHeavyBenchmark) {
            $textHeavy = Invoke-RestMethod -Method Post -Uri "$baseAddress/benchmark/text-heavy?requests=$TextHeavyRequests&concurrency=$TieredConcurrency"
            if ($TieredSummaryOnly) {
                $textHeavy.measurements | Select-Object engine, tier, requests, concurrency, transformsPerSecond, p50Microseconds, p95Microseconds, p99Microseconds, processorMilliseconds, normalizedProcessorPercent, managedAllocatedBytes, workerWorkingSetAfter
            }
            else {
                $textHeavy | ConvertTo-Json -Depth 6
            }
        }
        if ($ResultHeavyBenchmark) {
            $resultHeavy = Invoke-RestMethod -Method Post -Uri "$baseAddress/benchmark/result-heavy?requests=$ResultHeavyRequests&concurrency=$TieredConcurrency"
            if ($TieredSummaryOnly) {
                $resultHeavy.measurements | Select-Object engine, tier, requests, concurrency, transformsPerSecond, p50Microseconds, p95Microseconds, p99Microseconds, processorMilliseconds, normalizedProcessorPercent, managedAllocatedBytes, workerWorkingSetAfter
            }
            else {
                $resultHeavy | ConvertTo-Json -Depth 6
            }
        }
        if ($NativeBoundaryBreakdown) {
            for ($boundaryRun = 1; $boundaryRun -le $MeasurementRuns; $boundaryRun++) {
                $boundary = Invoke-RestMethod -Method Post -Uri "$baseAddress/benchmark/native-boundary-breakdown?requests=$TieredRequests"
                $boundary.measurements | Select-Object `
                    @{Name='Run'; Expression={$boundaryRun}}, tier, requests, `
                    @{Name='DirectPerSecond'; Expression={$_.direct.transformsPerSecond}}, `
                    @{Name='PoolPerSecond'; Expression={$_.oneSlotPool.transformsPerSecond}}, `
                    observedPoolOverheadMicroseconds, `
                    @{Name='TransformExportMicroseconds'; Expression={$_.instrumentedDirectMeans.transformExportMicroseconds}}, `
                    @{Name='CopyMicroseconds'; Expression={$_.instrumentedDirectMeans.outcomeCopyMicroseconds}}, `
                    @{Name='DecodeMicroseconds'; Expression={$_.instrumentedDirectMeans.resultDecodingMicroseconds}}, `
                    @{Name='ReleaseMicroseconds'; Expression={$_.instrumentedDirectMeans.outcomeReleaseMicroseconds}}, `
                    @{Name='InstrumentedTotalMicroseconds'; Expression={$_.instrumentedDirectMeans.instrumentedTotalMicroseconds}}
            }
        }
        if ($IsolatedBoundaryBreakdown) {
            for ($boundaryRun = 1; $boundaryRun -le $MeasurementRuns; $boundaryRun++) {
                $boundary = Invoke-RestMethod -Method Post -Uri "$baseAddress/benchmark/isolated-boundary-breakdown?requests=$TieredRequests"
                $boundary.measurements | Select-Object `
                    @{Name='Run'; Expression={$boundaryRun}}, tier, requests, `
                    @{Name='OrdinaryPerSecond'; Expression={$_.ordinary.transformsPerSecond}}, `
                    @{Name='OrdinaryMeanMicroseconds'; Expression={$_.ordinary.meanMicroseconds}}, `
                    @{Name='RequestWriteMicroseconds'; Expression={$_.instrumentedMeans.requestWriteMicroseconds}}, `
                    @{Name='RequestFlushMicroseconds'; Expression={$_.instrumentedMeans.requestFlushMicroseconds}}, `
                    @{Name='WorkerDecodeMicroseconds'; Expression={$_.instrumentedMeans.workerDecodeMicroseconds}}, `
                    @{Name='WorkerQueueMicroseconds'; Expression={$_.instrumentedMeans.workerQueueMicroseconds}}, `
                    @{Name='WorkerExecutionMicroseconds'; Expression={$_.instrumentedMeans.workerExecutionMicroseconds}}, `
                    @{Name='UnattributedRoundTripMicroseconds'; Expression={$_.instrumentedMeans.unattributedRoundTripMicroseconds}}, `
                    @{Name='InstrumentedTotalMicroseconds'; Expression={$_.instrumentedMeans.instrumentedTotalMicroseconds}}
            }
        }
        if ($IsolatedBatchBenchmark) {
            for ($batchRun = 1; $batchRun -le $MeasurementRuns; $batchRun++) {
                $orderOffset = $batchRun - 1
                $batch = Invoke-RestMethod -Method Post -Uri "$baseAddress/benchmark/isolated-batch?requests=$TieredRequests&orderOffset=$orderOffset"
                $batch.measurements | Select-Object `
                    @{Name='Run'; Expression={$batchRun}}, tier, mode, batchSize, requests, transactions, `
                    transformsPerSecond, meanMicrosecondsPerMember, managedAllocatedBytesPerMember, `
                    transactionP50Microseconds, transactionP95Microseconds, transactionP99Microseconds
            }
        }
        if ($IsolatedBatchWorkerSweep) {
            $sweepMeasurements = [System.Collections.Generic.List[object]]::new()
            for ($sweepRun = 1; $sweepRun -le $MeasurementRuns; $sweepRun++) {
                $orderOffset = $sweepRun - 1
                $sweep = Invoke-RestMethod -Method Post -Uri "$baseAddress/benchmark/isolated-batch-worker-sweep?members=$BatchSweepMembers&orderOffset=$orderOffset"
                foreach ($measurement in $sweep.measurements) {
                    $measurement | Add-Member -NotePropertyName Run -NotePropertyValue $sweepRun
                    $sweepMeasurements.Add($measurement)
                }
            }
            if ($TieredSummaryOnly) {
                'Tier|Workers|Batch|Runs|MedianTps|MinTps|MaxTps|MedianCpuMs|MedianBusyCores|MedianWorkingSetBytes|MedianManagedBytesPerMember|MaxResponseFrameBytes|AmbiguousPerWorkerLoss|AggregateAmbiguous'
                $sweepMeasurements | Group-Object tier, workers, batchSize |
                    Sort-Object `
                        @{ Expression = { [int]($_.Group[0].tier -replace '[^0-9]', '') } }, `
                        @{ Expression = { [int]$_.Group[0].workers } }, `
                        @{ Expression = { [int]$_.Group[0].batchSize } } |
                    ForEach-Object {
                    $first = $_.Group[0]
                    $throughput = @($_.Group.transformsPerSecond)
                    $minimumThroughput = ($throughput | Measure-Object -Minimum).Minimum
                    $maximumThroughput = ($throughput | Measure-Object -Maximum).Maximum
                    @(
                        $first.tier
                        $first.workers
                        $first.batchSize
                        $_.Count
                        ('{0:F0}' -f (Get-Median $throughput))
                        ('{0:F0}' -f $minimumThroughput)
                        ('{0:F0}' -f $maximumThroughput)
                        ('{0:F3}' -f (Get-Median @($_.Group.aggregateWorkerCpuMilliseconds)))
                        ('{0:F3}' -f (Get-Median @($_.Group.effectiveBusyCores)))
                        ('{0:F0}' -f (Get-Median @($_.Group.aggregateWorkingSetAfterBytes)))
                        ('{0:F1}' -f (Get-Median @($_.Group.managedAllocatedBytesPerMember)))
                        $first.maximumResponseFrameBytes
                        $first.maximumAmbiguousMembersPerWorkerLoss
                        $first.maximumAggregateAmbiguousMembers
                    ) -join '|'
                }
            }
            else {
                $sweepMeasurements | Select-Object Run, tier, workers, batchSize, members, transactions, `
                    transformsPerSecond, aggregateWorkerCpuMilliseconds, effectiveBusyCores, `
                    aggregateWorkingSetBeforeBytes, aggregateWorkingSetAfterBytes, `
                    managedAllocatedBytesPerMember, totalRequestWireBytes, totalResponseWireBytes, `
                    maximumRequestFrameBytes, maximumResponseFrameBytes, maximumOutstandingMembers, `
                    maximumAmbiguousMembersPerWorkerLoss, maximumAggregateAmbiguousMembers
            }
        }
        if ($IsolatedBatchResultPressure) {
            $resultPressureMeasurements = [System.Collections.Generic.List[object]]::new()
            for ($pressureRun = 1; $pressureRun -le $MeasurementRuns; $pressureRun++) {
                $orderOffset = $pressureRun - 1
                $pressure = Invoke-RestMethod -Method Post -Uri "$baseAddress/benchmark/isolated-batch-result-pressure?members=$BatchResultMembers&orderOffset=$orderOffset"
                foreach ($measurement in $pressure.measurements) {
                    $measurement | Add-Member -NotePropertyName Run -NotePropertyValue $pressureRun
                    $resultPressureMeasurements.Add($measurement)
                }
            }
            if ($TieredSummaryOnly) {
                'Tier|ResultBytes|Workers|Batch|Delivery|Runs|MedianTps|MinTps|MaxTps|MedianFirstP50Ms|MedianFirstP95Ms|MedianFinalP50Ms|MedianFinalP95Ms|MedianFinalP99Ms|MedianWorkingSetBytes|MedianManagedBytesPerMember|MaxResponseFrameBytes|AmbiguousPerWorkerLoss|AggregateAmbiguous'
                $resultPressureMeasurements | Group-Object tier, workers, batchSize, delivery |
                    Sort-Object `
                        @{ Expression = { [int]($_.Group[0].tier -replace '[^0-9]', '') } }, `
                        @{ Expression = { [int]$_.Group[0].workers } }, `
                        @{ Expression = { [int]$_.Group[0].batchSize } }, `
                        @{ Expression = { $_.Group[0].delivery } } |
                    ForEach-Object {
                    $first = $_.Group[0]
                    $throughput = @($_.Group.transformsPerSecond)
                    @(
                        $first.tier
                        $first.resultBytes
                        $first.workers
                        $first.batchSize
                        $first.delivery
                        $_.Count
                        ('{0:F0}' -f (Get-Median $throughput))
                        ('{0:F0}' -f (($throughput | Measure-Object -Minimum).Minimum))
                        ('{0:F0}' -f (($throughput | Measure-Object -Maximum).Maximum))
                        ('{0:F3}' -f (Get-Median @($_.Group.firstOutcomeP50Milliseconds)))
                        ('{0:F3}' -f (Get-Median @($_.Group.firstOutcomeP95Milliseconds)))
                        ('{0:F3}' -f (Get-Median @($_.Group.finalOutcomeP50Milliseconds)))
                        ('{0:F3}' -f (Get-Median @($_.Group.finalOutcomeP95Milliseconds)))
                        ('{0:F3}' -f (Get-Median @($_.Group.finalOutcomeP99Milliseconds)))
                        ('{0:F0}' -f (Get-Median @($_.Group.aggregateWorkingSetAfterBytes)))
                        ('{0:F1}' -f (Get-Median @($_.Group.managedAllocatedBytesPerMember)))
                        $first.maximumResponseFrameBytes
                        $first.maximumAmbiguousMembersPerWorkerLoss
                        $first.maximumAggregateAmbiguousMembers
                    ) -join '|'
                }
            }
            else {
                $resultPressureMeasurements | Select-Object Run, tier, resultBytes, workers, batchSize, delivery, `
                    members, transactions, transformsPerSecond, firstOutcomeP50Milliseconds, `
                    firstOutcomeP95Milliseconds, firstOutcomeP99Milliseconds, `
                    finalOutcomeP50Milliseconds, finalOutcomeP95Milliseconds, finalOutcomeP99Milliseconds, `
                    aggregateWorkingSetBeforeBytes, aggregateWorkingSetAfterBytes, `
                    managedAllocatedBytesPerMember, maximumResponseFrameBytes, `
                    maximumAmbiguousMembersPerWorkerLoss, maximumAggregateAmbiguousMembers
            }
        }
    }
    finally {
        if (-not $server.HasExited) {
            Stop-Process -Id $server.Id
            $server.WaitForExit()
        }
    }
}
finally {
    Pop-Location
}
