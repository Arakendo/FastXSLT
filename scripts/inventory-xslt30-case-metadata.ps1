[CmdletBinding()]
param(
    [switch]$IncludeMetadataShapes
)

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

function Add-Count {
    param(
        [Parameter(Mandatory)]
        [hashtable]$Counts,

        [Parameter(Mandatory)]
        [string]$Name
    )

    if ($Counts.ContainsKey($Name)) {
        $Counts[$Name]++
    }
    else {
        $Counts[$Name] = 1
    }
}

function Convert-Counts {
    param(
        [Parameter(Mandatory)]
        [hashtable]$Counts
    )

    return @(
        $Counts.GetEnumerator() |
            Sort-Object -Property Name |
            ForEach-Object {
                [pscustomobject]@{
                    name = $_.Name
                    count = $_.Value
                }
            }
    )
}

function Get-ElementFingerprint {
    param(
        [Parameter(Mandatory)]
        [System.Xml.Linq.XElement]$Element
    )

    $attributes = @(
        $Element.Attributes() |
            Sort-Object { $_.Name.LocalName } |
            ForEach-Object { "$($_.Name.LocalName)=$($_.Value)" }
    )
    if ($attributes.Count -eq 0) {
        return $Element.Name.LocalName
    }

    return "$($Element.Name.LocalName)($($attributes -join ','))"
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

$repositoryRoot = Split-Path -Parent $PSScriptRoot
& "$PSScriptRoot/check-conformance-sources.ps1" -Quiet

$suiteRevision = '6f8fd9e966ae74a251a2604abef9d904c7bc5c9b'
$suiteRoot = [System.IO.Path]::GetFullPath((Join-Path $repositoryRoot 'vendor/xslt30-test'))
$suitePrefix = $suiteRoot.TrimEnd(
    [System.IO.Path]::DirectorySeparatorChar,
    [System.IO.Path]::AltDirectorySeparatorChar
) + [System.IO.Path]::DirectorySeparatorChar
$catalogPath = Join-Path $suiteRoot 'catalog.xml'
$references = @(Get-TestSetReferences -CatalogPath $catalogPath | Sort-Object -Unique)

$dependencyKinds = @{}
$specValues = @{}
$topLevelAssertions = @{}
$assertionElements = @{}
$environmentBindings = @{}
$metadataShapes = @{}
$stylesheetFiles = [System.Collections.Generic.HashSet[string]]::new(
    [System.StringComparer]::Ordinal
)
$caseCount = 0
$stylesheetReferenceCount = 0

foreach ($reference in $references) {
    $testSetPath = [System.IO.Path]::GetFullPath((Join-Path $suiteRoot $reference))
    if (-not $testSetPath.StartsWith($suitePrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Test-set reference escapes the pinned suite root: $reference"
    }
    if (-not (Test-Path -LiteralPath $testSetPath -PathType Leaf)) {
        throw "Pinned XSLT30 test set is missing: $reference"
    }

    $reader = New-SafeXmlReader -Path $testSetPath
    try {
        $document = [System.Xml.Linq.XDocument]::Load(
            $reader,
            [System.Xml.Linq.LoadOptions]::None
        )
    }
    finally {
        $reader.Dispose()
    }

    $testCases = @(
        $document.Descendants() |
            Where-Object { $_.Name.LocalName -eq 'test-case' }
    )
    foreach ($testCase in $testCases) {
        $caseCount++

        $environment = $testCase.Elements() |
            Where-Object { $_.Name.LocalName -eq 'environment' } |
            Select-Object -First 1
        if ($null -eq $environment) {
            $environmentBinding = 'absent'
        }
        elseif ($null -ne $environment.Attribute('ref')) {
            $environmentBinding = 'referenced'
        }
        else {
            $environmentBinding = 'inline'
        }
        Add-Count -Counts $environmentBindings -Name $environmentBinding

        $dependencyLabels = [System.Collections.Generic.List[string]]::new()
        $dependencies = $testCase.Elements() |
            Where-Object { $_.Name.LocalName -eq 'dependencies' } |
            Select-Object -First 1
        if ($null -ne $dependencies) {
            foreach ($dependency in $dependencies.Elements()) {
                Add-Count -Counts $dependencyKinds -Name $dependency.Name.LocalName
                $dependencyLabels.Add((Get-ElementFingerprint -Element $dependency))
                if ($dependency.Name.LocalName -eq 'spec') {
                    $valueAttribute = $dependency.Attribute('value')
                    $value = if ($null -eq $valueAttribute) {
                        ''
                    }
                    else {
                        $valueAttribute.Value
                    }
                    if ([string]::IsNullOrWhiteSpace($value)) {
                        $value = '<missing>'
                    }
                    Add-Count -Counts $specValues -Name $value
                }
            }
        }

        $topAssertions = [System.Collections.Generic.List[string]]::new()
        $result = $testCase.Elements() |
            Where-Object { $_.Name.LocalName -eq 'result' } |
            Select-Object -First 1
        if ($null -ne $result) {
            foreach ($assertion in $result.Elements()) {
                Add-Count -Counts $topLevelAssertions -Name $assertion.Name.LocalName
                $topAssertions.Add($assertion.Name.LocalName)
            }
            foreach ($assertion in $result.Descendants()) {
                Add-Count -Counts $assertionElements -Name $assertion.Name.LocalName
            }
        }

        $caseStylesheets = @(
            $testCase.Descendants() |
                Where-Object { $_.Name.LocalName -eq 'stylesheet' }
        )
        foreach ($stylesheet in $caseStylesheets) {
            $fileAttribute = $stylesheet.Attribute('file')
            $file = if ($null -eq $fileAttribute) {
                ''
            }
            else {
                $fileAttribute.Value
            }
            if (-not [string]::IsNullOrWhiteSpace($file)) {
                $stylesheetReferenceCount++
                $relativeDirectory = [System.IO.Path]::GetDirectoryName($reference)
                $normalized = [System.IO.Path]::GetFullPath(
                    (Join-Path (Join-Path $suiteRoot $relativeDirectory) $file)
                )
                if (-not $normalized.StartsWith(
                    $suitePrefix,
                    [System.StringComparison]::OrdinalIgnoreCase
                )) {
                    throw "Stylesheet reference escapes the pinned suite root: $reference -> $file"
                }
                if (-not (Test-Path -LiteralPath $normalized -PathType Leaf)) {
                    throw "Stylesheet reference is missing: $reference -> $file"
                }
                $relativeStylesheet = ([System.IO.Path]::GetRelativePath(
                    $suiteRoot,
                    $normalized
                )).Replace([System.IO.Path]::DirectorySeparatorChar, '/')
                [void]$stylesheetFiles.Add($relativeStylesheet)
            }
        }

        $dependencyShape = if ($dependencyLabels.Count -eq 0) {
            '<none>'
        }
        else {
            (@($dependencyLabels | Sort-Object -Unique) -join ';')
        }
        $assertionShape = if ($topAssertions.Count -eq 0) {
            '<none>'
        }
        else {
            (@($topAssertions | Sort-Object -Unique) -join '+')
        }
        $stylesheetShape = if ($caseStylesheets.Count -eq 0) {
            'none'
        }
        elseif ($caseStylesheets.Count -eq 1) {
            'one'
        }
        else {
            'multiple'
        }
        $shape = "dependencies=$dependencyShape|environment=$environmentBinding|stylesheets=$stylesheetShape|assertions=$assertionShape"
        Add-Count -Counts $metadataShapes -Name $shape
    }
}

if ($caseCount -ne 14600) {
    throw "Expected 14,600 pinned XSLT30 cases but inventoried $caseCount."
}
if ($stylesheetReferenceCount -ne 9663) {
    throw "Expected 9,663 pinned stylesheet references but inventoried $stylesheetReferenceCount."
}
if ($stylesheetFiles.Count -ne 7646) {
    throw "Expected 7,646 distinct pinned stylesheet files but inventoried $($stylesheetFiles.Count)."
}
if ($metadataShapes.Count -ne 564) {
    throw "Expected 564 pinned metadata shapes but inventoried $($metadataShapes.Count)."
}

$inventory = [ordered]@{
    suite = 'XSLT 3.0'
    revision = $suiteRevision
    test_sets = $references.Count
    test_cases = $caseCount
    stylesheet_references = $stylesheetReferenceCount
    distinct_stylesheet_files = $stylesheetFiles.Count
    dependency_kinds = Convert-Counts -Counts $dependencyKinds
    spec_values = Convert-Counts -Counts $specValues
    environment_bindings = Convert-Counts -Counts $environmentBindings
    top_level_assertions = Convert-Counts -Counts $topLevelAssertions
    assertion_elements = Convert-Counts -Counts $assertionElements
    metadata_shape_count = $metadataShapes.Count
}
if ($IncludeMetadataShapes) {
    $inventory.metadata_shapes = Convert-Counts -Counts $metadataShapes
}

[pscustomobject]$inventory | ConvertTo-Json -Depth 6
