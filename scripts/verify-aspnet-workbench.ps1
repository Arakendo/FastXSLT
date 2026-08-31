[CmdletBinding()]
param(
    [int]$Port = 5087,
    [int]$MeasurementRequests = 1000,
    [int]$MeasurementRuns = 3,
    [switch]$LocalSaxonCs,
    [switch]$TieredBenchmark,
    [switch]$TieredSummaryOnly,
    [switch]$OperationalExperiments,
    [switch]$NativeRegistryPressure,
    [switch]$RegistrySummaryOnly,
    [int]$TieredRequests = 250,
    [int]$TieredConcurrency = 4,
    [int]$RegistryItems = 500,
    [int]$RegistryConcurrency = 4,
    [int]$RegistryGenerations = 2,
    [int]$RegistryDelayedOutcomes = 64
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
    cargo build --release -p fastxslt-worker -p fastxslt-dotnet-workbench
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
    $nativeLibraryName = if ($IsWindows) { 'fastxslt_dotnet_workbench.dll' } elseif ($IsMacOS) { 'libfastxslt_dotnet_workbench.dylib' } else { 'libfastxslt_dotnet_workbench.so' }
    $nativeLibrary = Join-Path $repositoryRoot "target/release/$nativeLibraryName"
    $managedOutput = Join-Path $repositoryRoot 'workbenches/FastXSLT.AspNet.Workbench/bin/Release/net8.0'
    Copy-Item -LiteralPath $nativeLibrary -Destination $managedOutput -Force

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
            $registryPressureUri = "$baseAddress/experiment/native-registry-pressure?items=$RegistryItems&concurrency=$RegistryConcurrency&generations=$RegistryGenerations&delayedOutcomes=$RegistryDelayedOutcomes"
            $registryPressure = Invoke-RestMethod -Method Post -Uri $registryPressureUri
            $baseline = $registryPressure.checkpoints[0].registry
            if (-not $registryPressure.logicalRegistryReturnedToBaseline -or
                $registryPressure.abandonedHandles -ne 0 -or
                $registryPressure.legitimateHighWater.engineHandles -ne ($baseline.engineHandles + ($RegistryConcurrency * $RegistryGenerations)) -or
                $registryPressure.legitimateHighWater.outcomeHandles -ne ($baseline.outcomeHandles + $RegistryDelayedOutcomes) -or
                $registryPressure.legitimateHighWater.outcomePayloadBytes -le $baseline.outcomePayloadBytes -or
                $registryPressure.settlement.Count -ne 6 -or
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
                    WorkingSetAfterOneSecond = $registryPressure.settlement[-1].checkpoint.workingSetBytes
                    PrivateBytesBaseline = $registryPressure.checkpoints[0].privateMemoryBytes
                    PrivateBytesPeak = ($allPrivateBytes | Measure-Object -Maximum).Maximum
                    PrivateBytesAfterOneSecond = $registryPressure.settlement[-1].checkpoint.privateMemoryBytes
                }
            }
            else {
                $registryPressure | ConvertTo-Json -Depth 8
            }
        }
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
