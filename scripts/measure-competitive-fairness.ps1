[CmdletBinding()]
param(
    [ValidateRange(3, 31)]
    [int]$Samples = 7,
    [ValidateSet('net8.0', 'net10.0')]
    [string]$TargetFramework = 'net10.0',
    [ValidateRange(1, 10000)]
    [int]$TieredRequests = 250,
    [ValidateRange(1, 5)]
    [int]$WithinProcessRounds = 3,
    [ValidateRange(1, 8)]
    [int]$TieredConcurrency = 4,
    [ValidateRange(1024, 60000)]
    [int]$StartingPort = 5187,
    [int]$OrderSeedBase = 31000,
    [switch]$LocalSaxonCs,
    [ValidateSet('None', 'ByteStream', 'TextWriter')]
    [string]$SelectedSaxonDestination = 'None'
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
        TieredBenchmark = $true
        TieredOnly = $true
        TieredSummaryOnly = $true
        TieredRequests = $TieredRequests
        TieredConcurrency = $TieredConcurrency
        TieredOrderSeedBase = $OrderSeedBase + ($sample * 100)
        MeasurementRuns = $WithinProcessRounds
    }
    if ($LocalSaxonCs) {
        $arguments.LocalSaxonCs = $true
    }
    Write-Host "Fresh-process competitive sample $sample/$Samples"
    $run = & $runner @arguments
    foreach ($item in $run) {
        if ($null -ne $item.tier -and
            ($null -ne $item.transformsPerSecond -or $null -ne $item.totalCalls)) {
            $item | Add-Member -NotePropertyName FreshProcessSample -NotePropertyValue $sample
            $observations.Add($item)
        }
    }
}

$measurements = @($observations | Where-Object { $null -ne $_.transformsPerSecond })
$measurementSummary = $measurements |
    Group-Object engine, tier, concurrency |
    ForEach-Object {
        [double[]]$rates = @($_.Group |
            Group-Object FreshProcessSample |
            ForEach-Object {
                Get-Percentile ([double[]]@($_.Group.transformsPerSecond)) 0.50
            })
        $mean = ($rates | Measure-Object -Average).Average
        $standardDeviation = Get-SampleStandardDeviation $rates $mean
        $margin = 1.96 * $standardDeviation / [Math]::Sqrt($rates.Count)
        [pscustomobject]@{
            Kind = 'measurement-distribution'
            Engine = $_.Group[0].engine
            Tier = $_.Group[0].tier
            Concurrency = $_.Group[0].concurrency
            Samples = $rates.Count
            RoundsPerProcess = $WithinProcessRounds
            MinimumTransformsPerSecond = ($rates | Measure-Object -Minimum).Minimum
            P25TransformsPerSecond = Get-Percentile $rates 0.25
            MedianTransformsPerSecond = Get-Percentile $rates 0.50
            P75TransformsPerSecond = Get-Percentile $rates 0.75
            MaximumTransformsPerSecond = ($rates | Measure-Object -Maximum).Maximum
            MeanTransformsPerSecond = $mean
            SampleStandardDeviation = $standardDeviation
            CoefficientOfVariation = if ($mean -eq 0) { 0 } else { $standardDeviation / $mean }
            ApproximateMean95PercentLower = $mean - $margin
            ApproximateMean95PercentUpper = $mean + $margin
            MinimumElapsedMilliseconds =
                ($_.Group.elapsedMilliseconds | Measure-Object -Minimum).Minimum
            RequestCounts = @(($_.Group.requests | Sort-Object -Unique))
            DistributionStable = if ($mean -eq 0) {
                $true
            }
            else {
                ($standardDeviation / $mean) -le 0.20
            }
            AchievedConcurrency = @(($_.Group.achievedConcurrencyHighWater | Sort-Object -Unique))
            MeasurementPositions = @(($_.Group.measurementPosition | Sort-Object))
            OrderSeeds = @(($_.Group.orderSeed | Sort-Object))
        }
    }

$warmups = @($observations | Where-Object { $null -ne $_.totalCalls })
$warmupSummary = $warmups |
    Group-Object engine, tier |
    ForEach-Object {
        $processes = @($_.Group | Group-Object FreshProcessSample)
        $stable = @($processes | Where-Object {
            @($_.Group | Where-Object { -not $_.stabilized }).Count -eq 0
        }).Count
        [pscustomobject]@{
            Kind = 'warmup-distribution'
            Engine = $_.Group[0].engine
            Tier = $_.Group[0].tier
            Samples = $processes.Count
            RoundsPerProcess = $WithinProcessRounds
            StabilizedSamples = $stable
            MaximumWarmupCalls = ($_.Group.totalCalls | Measure-Object -Maximum).Maximum
            MaximumFinalRelativeMedianDrift =
                ($_.Group.finalRelativeMedianDrift | Measure-Object -Maximum).Maximum
            MaximumFinalRelativeMedianAbsoluteDeviation =
                ($_.Group.finalRelativeMedianAbsoluteDeviation | Measure-Object -Maximum).Maximum
            WindowThroughputsPerSecond = @($_.Group | ForEach-Object {
                [pscustomobject]@{
                    FreshProcessSample = $_.FreshProcessSample
                    WithinProcessRound = $_.Run
                    Values = @($_.windowThroughputsPerSecond)
                }
            })
        }
    }

$allRequestedConcurrencyReached = @(
    $measurementSummary | Where-Object {
        $_.Concurrency -gt 1 -and $_.AchievedConcurrency -notcontains $_.Concurrency
    }).Count -eq 0
$saxonSelectionDeclared = -not $LocalSaxonCs -or $SelectedSaxonDestination -ne 'None'
$isPublicationLane = {
    param($Lane)
    if ($Lane.Engine -like 'SaxonCS*') {
        return ($SelectedSaxonDestination -eq 'ByteStream' -and $Lane.Engine -like '*byte*stream*') -or
            ($SelectedSaxonDestination -eq 'TextWriter' -and $Lane.Engine -like '*TextWriter*')
    }
    if ($Lane.Engine -like '*oracle*') {
        return $false
    }
    return $true
}
$publicationWarmups = @($warmupSummary | Where-Object { & $isPublicationLane $_ })
$publicationMeasurements = @($measurementSummary | Where-Object { & $isPublicationLane $_ })
$selectedSaxonWarmups = @($publicationWarmups | Where-Object { $_.Engine -like 'SaxonCS*' })
$selectedSaxonMeasurements = @($publicationMeasurements | Where-Object { $_.Engine -like 'SaxonCS*' })
$saxonSelectionAvailable = -not $LocalSaxonCs -or
    ($selectedSaxonWarmups.Count -gt 0 -and $selectedSaxonMeasurements.Count -gt 0)
$saxonSelectionEstablished = $saxonSelectionDeclared -and $saxonSelectionAvailable
$allWarmupsStabilized = $saxonSelectionEstablished -and @(
    $publicationWarmups | Where-Object { $_.StabilizedSamples -ne $_.Samples }).Count -eq 0
$minimumPublicationSamplesMet = $Samples -ge 7
$allSelectedDistributionsStable = $saxonSelectionEstablished -and @(
    $publicationMeasurements | Where-Object { -not $_.DistributionStable }).Count -eq 0
$allSelectedMeasurementDurationsMet = $saxonSelectionEstablished -and @(
    $publicationMeasurements | Where-Object {
        $_.MinimumElapsedMilliseconds -lt 250
    }).Count -eq 0

[pscustomobject]@{
    Method = 'independent fresh processes with seeded whole-lane rotation'
    Samples = $Samples
    TargetFramework = $TargetFramework
    TieredRequests = $TieredRequests
    WithinProcessRounds = $WithinProcessRounds
    MaximumConcurrency = $TieredConcurrency
    LocalSaxonCs = [bool]$LocalSaxonCs
    SelectedSaxonDestination = $SelectedSaxonDestination
    SaxonSelectionDeclared = $saxonSelectionDeclared
    SaxonSelectionAvailable = $saxonSelectionAvailable
    SaxonSelectionEstablished = $saxonSelectionEstablished
    MinimumPublicationSamplesMet = $minimumPublicationSamplesMet
    AllRequestedConcurrencyReached = $allRequestedConcurrencyReached
    AllWarmupsStabilized = $allWarmupsStabilized
    AllSelectedDistributionsStable = $allSelectedDistributionsStable
    AllSelectedMeasurementDurationsMet = $allSelectedMeasurementDurationsMet
    PublicationEligible =
        $minimumPublicationSamplesMet -and
        $allRequestedConcurrencyReached -and
        $allWarmupsStabilized -and
        $allSelectedDistributionsStable -and
        $allSelectedMeasurementDurationsMet -and
        $saxonSelectionEstablished
    Measurements = @($measurementSummary)
    Warmups = @($warmupSummary)
}
