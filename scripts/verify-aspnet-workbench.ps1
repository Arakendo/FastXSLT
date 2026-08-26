[CmdletBinding()]
param(
    [int]$Port = 5087,
    [int]$MeasurementRequests = 1000,
    [int]$MeasurementRuns = 3,
    [switch]$LocalSaxonCs,
    [switch]$TieredBenchmark,
    [switch]$TieredSummaryOnly,
    [switch]$OperationalExperiments,
    [int]$TieredRequests = 250,
    [int]$TieredConcurrency = 4
)

$ErrorActionPreference = 'Stop'
if ($MeasurementRuns -lt 1) {
    throw 'MeasurementRuns must be at least 1.'
}
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$project = Join-Path $repositoryRoot 'workbenches/FastXSLT.AspNet.Workbench/FastXSLT.AspNet.Workbench.csproj'
$workbenchDirectory = Join-Path $repositoryRoot '.workbench'
$stdoutLog = Join-Path $workbenchDirectory 'aspnet-stdout.log'
$stderrLog = Join-Path $workbenchDirectory 'aspnet-stderr.log'
$baseAddress = "http://127.0.0.1:$Port"

New-Item -ItemType Directory -Path $workbenchDirectory -Force | Out-Null

Push-Location $repositoryRoot
try {
    cargo build --release -p fastxslt-worker
    if ($LASTEXITCODE -ne 0) {
        throw "Rust worker build failed with exit code $LASTEXITCODE"
    }
    $dotnetBuildArguments = @('build', $project, '--configuration', 'Release')
    if ($LocalSaxonCs) {
        $dotnetBuildArguments += '-p:EnableLocalSaxonCs=true'
    }
    dotnet @dotnetBuildArguments
    if ($LASTEXITCODE -ne 0) {
        throw "ASP.NET workbench build failed with exit code $LASTEXITCODE"
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

        $result = Invoke-WebRequest -Method Post -Uri "$baseAddress/transform/smoke-001"
        $expected = '<?xml version="1.0" encoding="UTF-8"?><out>36.02</out>'
        if ($result.StatusCode -ne 200 -or $result.Content -cne $expected) {
            throw "Unexpected transform response: $($result.StatusCode) $($result.Content)"
        }
        if ($health.dotNetXslt1ExactStylesheetExecuted) {
            throw 'XslCompiledTransform unexpectedly executed the exact XSLT 2.0 stylesheet.'
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
        for ($run = 1; $run -le $MeasurementRuns; $run++) {
            $fastXslt = Invoke-RestMethod -Method Post -Uri "$baseAddress/measure?requests=$MeasurementRequests"
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
        if ($TieredBenchmark) {
            $tiered = Invoke-RestMethod -Method Post -Uri "$baseAddress/benchmark/tiers?requests=$TieredRequests&concurrency=$TieredConcurrency"
            if ($TieredSummaryOnly) {
                $tiered.measurements | Select-Object engine, tier, requests, concurrency, transformsPerSecond, p50Microseconds, p95Microseconds, p99Microseconds, processorMilliseconds, normalizedProcessorPercent, managedAllocatedBytes, workerWorkingSetAfter
            }
            else {
                $tiered | ConvertTo-Json -Depth 6
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
