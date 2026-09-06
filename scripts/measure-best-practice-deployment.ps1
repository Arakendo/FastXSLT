[CmdletBinding()]
param(
    [ValidateRange(3, 31)]
    [int]$Samples = 3,
    [ValidateRange(1, 5)]
    [int]$WithinProcessRounds = 3,
    [ValidateSet('net8.0', 'net10.0')]
    [string]$TargetFramework = 'net10.0',
    [ValidateRange(128, 20000)]
    [int]$BaseMembers = 4000,
    [ValidateRange(1, 8)]
    [int]$Concurrency = 4,
    [ValidateRange(1024, 60000)]
    [int]$StartingPort = 5587,
    [int]$OrderSeedBase = 51000,
    [switch]$LocalSaxonCs
)

$ErrorActionPreference = 'Stop'
if ($WithinProcessRounds % 2 -eq 0) {
    throw 'WithinProcessRounds must be odd so every process has an unambiguous median.'
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
for ($sample = 1; $sample -le $Samples; $sample++) {
    $arguments = @{
        Port = $StartingPort + $sample - 1
        TargetFramework = $TargetFramework
        TieredOnly = $true
        BestPracticeDeploymentBenchmark = $true
        TieredSummaryOnly = $true
        BestPracticeMembers = $BaseMembers
        TieredConcurrency = $Concurrency
        TieredOrderSeedBase = $OrderSeedBase + ($sample * 100)
        MeasurementRuns = $WithinProcessRounds
    }
    if ($LocalSaxonCs) {
        $arguments.LocalSaxonCs = $true
    }
    Write-Host "Fresh-process deployment sample $sample/$Samples"
    $run = & $runner @arguments
    foreach ($item in $run) {
        if ($null -ne $item.transformsPerSecond) {
            $item | Add-Member -NotePropertyName FreshProcessSample -NotePropertyValue $sample
            $observations.Add($item)
        }
    }
}

$summary = $observations |
    Group-Object engine, mode, tier, concurrency, batchSize |
    ForEach-Object {
        $group = $_.Group
        [double[]]$rates = @($group |
            Group-Object FreshProcessSample |
            ForEach-Object {
                Get-Percentile ([double[]]@($_.Group.transformsPerSecond)) 0.50
            })
        $mean = ($rates | Measure-Object -Average).Average
        $standardDeviation = Get-SampleStandardDeviation $rates $mean
        $first = [double[]]@($group.firstResultP50Microseconds)
        $final = [double[]]@($group.finalResultP50Microseconds)
        [pscustomobject]@{
            Engine = $group[0].engine
            Mode = $group[0].mode
            Tier = $group[0].tier
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
            MedianFirstResultP50Microseconds = Get-Percentile $first 0.50
            MedianFinalResultP50Microseconds = Get-Percentile $final 0.50
            MinimumElapsedMilliseconds =
                ($group.elapsedMilliseconds | Measure-Object -Minimum).Minimum
            AchievedConcurrency = @(($group.achievedConcurrencyHighWater | Sort-Object -Unique))
            ManagedAllocatedBytesPerMemberMedian =
                Get-Percentile ([double[]]@($group.managedAllocatedBytesPerMember)) 0.50
            WorkerWorkingSetAfterBytesMaximum =
                ($group.workerWorkingSetAfterBytes | Measure-Object -Maximum).Maximum
            RequestWireBytesMedian =
                Get-Percentile ([double[]]@($group.requestWireBytes)) 0.50
            ResponseWireBytesMedian =
                Get-Percentile ([double[]]@($group.responseWireBytes)) 0.50
            MaximumAmbiguousMembersPerWorkerLoss =
                ($group.maximumAmbiguousMembersPerWorkerLoss | Measure-Object -Maximum).Maximum
            MaximumAggregateAmbiguousMembers =
                ($group.maximumAggregateAmbiguousMembers | Measure-Object -Maximum).Maximum
            MeasurementPositions = @(($group.measurementPosition | Sort-Object))
            OrderSeeds = @(($group.orderSeed | Sort-Object))
        }
    }

$allConcurrencyReached = @($summary | Where-Object {
    $_.AchievedConcurrency -notcontains $Concurrency
}).Count -eq 0
$allDurationsMet = @($summary | Where-Object {
    $_.MinimumElapsedMilliseconds -lt 250
}).Count -eq 0
$allDistributionsStable = @($summary | Where-Object {
    $_.CoefficientOfVariation -gt 0.20
}).Count -eq 0

[pscustomobject]@{
    Family = 'best-practice deployment exploratory'
    Samples = $Samples
    WithinProcessRounds = $WithinProcessRounds
    TargetFramework = $TargetFramework
    BaseMembersAtLargestTier = $BaseMembers
    Concurrency = $Concurrency
    LocalSaxonCs = [bool]$LocalSaxonCs
    AllRequestedConcurrencyReached = $allConcurrencyReached
    AllMeasurementDurationsMet = $allDurationsMet
    AllDistributionsStable = $allDistributionsStable
    PublicationEligible = $false
    PublicationBlocker =
        'This first deployment family has only correctness warm-up; sustained per-lane convergence is not yet implemented.'
    Measurements = @($summary)
}
