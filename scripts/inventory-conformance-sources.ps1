[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

function New-SafeXmlReader {
    param(
        [Parameter(Mandatory)]
        [string]$Path
    )

    $settings = [System.Xml.XmlReaderSettings]::new()
    $settings.DtdProcessing = [System.Xml.DtdProcessing]::Prohibit
    $settings.XmlResolver = $null
    $settings.IgnoreComments = $true
    $settings.IgnoreProcessingInstructions = $true
    $settings.IgnoreWhitespace = $true

    return [System.Xml.XmlReader]::Create($Path, $settings)
}

function Get-TestSetReferences {
    param(
        [Parameter(Mandatory)]
        [string]$CatalogPath
    )

    $references = [System.Collections.Generic.List[string]]::new()
    $reader = New-SafeXmlReader -Path $CatalogPath

    try {
        while ($reader.Read()) {
            if (
                $reader.NodeType -eq [System.Xml.XmlNodeType]::Element -and
                $reader.Depth -eq 1 -and
                $reader.LocalName -eq 'test-set'
            ) {
                $file = $reader.GetAttribute('file')
                if ([string]::IsNullOrWhiteSpace($file)) {
                    throw "Catalog test-set entry has no file attribute: $CatalogPath"
                }

                $references.Add($file.Replace('/', [System.IO.Path]::DirectorySeparatorChar))
            }
        }
    }
    finally {
        $reader.Dispose()
    }

    return $references
}

function Measure-TestCases {
    param(
        [Parameter(Mandatory)]
        [string]$TestSetPath
    )

    $count = 0
    $reader = New-SafeXmlReader -Path $TestSetPath

    try {
        while ($reader.Read()) {
            if (
                $reader.NodeType -eq [System.Xml.XmlNodeType]::Element -and
                $reader.LocalName -eq 'test-case'
            ) {
                $count++
            }
        }
    }
    finally {
        $reader.Dispose()
    }

    return $count
}

$repositoryRoot = Split-Path -Parent $PSScriptRoot
& "$PSScriptRoot/check-conformance-sources.ps1" -Quiet

$suites = @(
    @{
        Name = 'QT3'
        RelativePath = 'vendor/qt3tests'
        Revision = '83993587711dbd5c18ed846385ec37d079d6e492'
    },
    @{
        Name = 'XSLT 3.0'
        RelativePath = 'vendor/xslt30-test'
        Revision = '6f8fd9e966ae74a251a2604abef9d904c7bc5c9b'
    }
)

$inventory = foreach ($suite in $suites) {
    $suiteRoot = Join-Path $repositoryRoot $suite.RelativePath
    $catalogPath = Join-Path $suiteRoot 'catalog.xml'
    $references = @(Get-TestSetReferences -CatalogPath $catalogPath)
    $distinctReferences = @($references | Sort-Object -Unique)
    $missing = [System.Collections.Generic.List[string]]::new()
    $testCaseCount = 0

    foreach ($reference in $distinctReferences) {
        $testSetPath = Join-Path $suiteRoot $reference
        if (-not (Test-Path -LiteralPath $testSetPath -PathType Leaf)) {
            $missing.Add($reference)
            continue
        }

        $testCaseCount += Measure-TestCases -TestSetPath $testSetPath
    }

    $duplicateCount = $references.Count - $distinctReferences.Count
    if ($references.Count -eq 0) {
        throw "$($suite.Name) catalog contains no test-set references."
    }

    if ($missing.Count -ne 0) {
        throw "$($suite.Name) catalog references $($missing.Count) missing test-set files."
    }

    if ($duplicateCount -ne 0) {
        throw "$($suite.Name) catalog contains $duplicateCount duplicate test-set references."
    }

    [pscustomobject]@{
        suite = $suite.Name
        revision = $suite.Revision
        catalog = "$($suite.RelativePath)/catalog.xml"
        test_set_references = $references.Count
        distinct_test_sets = $distinctReferences.Count
        test_cases = $testCaseCount
        missing_test_sets = $missing.Count
        duplicate_references = $duplicateCount
    }
}

$inventory | ConvertTo-Json -Depth 3
