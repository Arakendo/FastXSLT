[CmdletBinding()]
param(
    [ValidateRange(1, 9)]
    [int]$Samples = 3,
    [ValidateRange(1, 5)]
    [int]$WithinProcessRounds = 3,
    [ValidateSet('net8.0', 'net10.0')]
    [string]$TargetFramework = 'net10.0',
    [ValidateNotNullOrEmpty()]
    [int[]]$QueuedJobs = @(500, 5000),
    [ValidateNotNullOrEmpty()]
    [int[]]$Concurrency = @(1, 4, 8),
    [ValidateRange(1024, 60000)]
    [int]$StartingPort = 5787,
    [int]$OrderSeedBase = 61000,
    [switch]$LocalSaxonCs,
    [switch]$CompactSummary
)

$ErrorActionPreference = 'Stop'
if ($WithinProcessRounds % 2 -eq 0) {
    throw 'WithinProcessRounds must be odd so every process has an unambiguous median.'
}
foreach ($jobs in $QueuedJobs) {
    if ($jobs -lt 1 -or $jobs -gt 50000) {
        throw "QueuedJobs must contain only values from 1 through 50000; received $jobs."
    }
}
foreach ($workers in $Concurrency) {
    if ($workers -lt 1 -or $workers -gt 8) {
        throw "Concurrency must contain only values from 1 through 8; received $workers."
    }
}

$runner = Join-Path $PSScriptRoot 'verify-aspnet-workbench.ps1'

function Get-Percentile([double[]]$Values, [double]$Percentile) {
    $ordered = @($Values | Sort-Object)
    $index = [Math]::Ceiling($Percentile * $ordered.Count) - 1
    return $ordered[[Math]::Clamp($index, 0, $ordered.Count - 1)]
}

function Get-SampleStandardDeviation([double[]]$Values, [double]$Mean) {
    if ($Values.Count -lt 2) {
        return 0
    }
    $sum = 0.0
    foreach ($value in $Values) {
        $sum += [Math]::Pow($value - $Mean, 2)
    }
    return [Math]::Sqrt($sum / ($Values.Count - 1))
}

$observations = [System.Collections.Generic.List[object]]::new()
$processIndex = 0
foreach ($workers in $Concurrency) {
    foreach ($jobs in $QueuedJobs) {
        for ($sample = 1; $sample -le $Samples; $sample++) {
            $processIndex++
            $arguments = @{
                Port = $StartingPort + $processIndex - 1
                TargetFramework = $TargetFramework
                TieredOnly = $true
                BestPracticeDeploymentBenchmark = $true
                TieredSummaryOnly = $true
                BestPracticeQueuedJobs = $jobs
                TieredConcurrency = $workers
                TieredOrderSeedBase =
                    $OrderSeedBase + ($workers * 10000) + ($jobs * 10) + ($sample * 100)
                MeasurementRuns = $WithinProcessRounds
            }
            if ($LocalSaxonCs) {
                $arguments.LocalSaxonCs = $true
            }
            Write-Host (
                "Fresh-process job-queue sample {0}/{1}: jobs={2}, concurrency={3}" -f
                $sample, $Samples, $jobs, $workers)
            $run = & $runner @arguments
            foreach ($item in $run) {
                if ($null -ne $item.transformsPerSecond) {
                    $item | Add-Member -NotePropertyName FreshProcessSample -NotePropertyValue $sample
                    $observations.Add($item)
                }
            }
        }
    }
}

$summary = $observations |
    Group-Object engine, mode, tier, queuedJobs, concurrency, batchSize |
    ForEach-Object {
        $group = $_.Group
        [double[]]$rates = @($group |
            Group-Object FreshProcessSample |
            ForEach-Object {
                Get-Percentile ([double[]]@($_.Group.transformsPerSecond)) 0.50
            })
        $mean = ($rates | Measure-Object -Average).Average
        $standardDeviation = Get-SampleStandardDeviation $rates $mean
        [pscustomobject]@{
            Engine = $group[0].engine
            Mode = $group[0].mode
            Tier = $group[0].tier
            ItemsPerTransform = $group[0].items
            QueuedJobs = $group[0].queuedJobs
            Concurrency = $group[0].concurrency
            BatchSize = $group[0].batchSize
            Samples = $rates.Count
            RoundsPerProcess = $WithinProcessRounds
            MinimumTransformsPerSecond = ($rates | Measure-Object -Minimum).Minimum
            MedianTransformsPerSecond = Get-Percentile $rates 0.50
            MaximumTransformsPerSecond = ($rates | Measure-Object -Maximum).Maximum
            CoefficientOfVariation = if ($mean -eq 0) { 0 } else {
                $standardDeviation / $mean
            }
            MedianElapsedMilliseconds =
                Get-Percentile ([double[]]@($group.elapsedMilliseconds)) 0.50
            MedianFirstResultP50Microseconds =
                Get-Percentile ([double[]]@($group.firstResultP50Microseconds)) 0.50
            MedianFinalResultP50Microseconds =
                Get-Percentile ([double[]]@($group.finalResultP50Microseconds)) 0.50
            MinimumAchievedConcurrency =
                ($group.achievedConcurrencyHighWater | Measure-Object -Minimum).Minimum
            ManagedAllocatedBytesPerMemberMedian =
                Get-Percentile ([double[]]@($group.managedAllocatedBytesPerMember)) 0.50
            MaximumAmbiguousMembersPerWorkerLoss =
                ($group.maximumAmbiguousMembersPerWorkerLoss | Measure-Object -Maximum).Maximum
            MaximumAggregateAmbiguousMembers =
                ($group.maximumAggregateAmbiguousMembers | Measure-Object -Maximum).Maximum
        }
    }

if ($CompactSummary) {
    'Items|Jobs|Workers|BestIsolatedBatch|BestIsolatedTps|NativeTps|MicrosoftTps|SaxonTps'
    $summary |
        Group-Object ItemsPerTransform, QueuedJobs, Concurrency |
        Sort-Object {
            [int]$_.Group[0].ItemsPerTransform
        }, {
            [int]$_.Group[0].QueuedJobs
        }, {
            [int]$_.Group[0].Concurrency
        } |
        ForEach-Object {
            $group = $_.Group
            $isolated = $group |
                Where-Object Engine -eq 'FastXSLT isolated incremental' |
                Sort-Object MedianTransformsPerSecond -Descending |
                Select-Object -First 1
            $native = $group |
                Where-Object Engine -eq 'FastXSLT native in-process' |
                Select-Object -First 1
            $microsoft = $group |
                Where-Object Engine -like 'Microsoft*' |
                Select-Object -First 1
            $saxon = $group |
                Where-Object Engine -like 'SaxonCS*' |
                Select-Object -First 1
            @(
                $group[0].ItemsPerTransform
                $group[0].QueuedJobs
                $group[0].Concurrency
                $isolated.BatchSize
                ('{0:F0}' -f $isolated.MedianTransformsPerSecond)
                ('{0:F0}' -f $native.MedianTransformsPerSecond)
                ('{0:F0}' -f $microsoft.MedianTransformsPerSecond)
                if ($null -eq $saxon) { '' } else {
                    '{0:F0}' -f $saxon.MedianTransformsPerSecond
                }
            ) -join '|'
        }
    return
}

[pscustomobject]@{
    Family = 'best-practice deployment with explicit job-queue axis'
    Samples = $Samples
    WithinProcessRounds = $WithinProcessRounds
    TargetFramework = $TargetFramework
    QueuedJobs = @($QueuedJobs)
    Concurrency = @($Concurrency)
    LocalSaxonCs = [bool]$LocalSaxonCs
    Axes = @(
        'items per transform',
        'queued transform jobs',
        'worker concurrency',
        'isolated transport batch members')
    PublicationEligible = $false
    PublicationBlocker =
        'Exploratory queue-scaling family; sustained convergence and representative consumer input distributions remain absent.'
    Measurements = @($summary)
}
