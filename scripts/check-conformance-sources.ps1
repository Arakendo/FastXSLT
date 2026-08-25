[CmdletBinding()]
param(
    [switch]$Quiet
)

$ErrorActionPreference = 'Stop'

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$suites = @(
    @{
        Name = 'W3C QT3'
        RelativePath = 'vendor/qt3tests'
        Revision = '83993587711dbd5c18ed846385ec37d079d6e492'
        Catalog = 'catalog.xml'
    },
    @{
        Name = 'W3C XSLT 3.0'
        RelativePath = 'vendor/xslt30-test'
        Revision = '6f8fd9e966ae74a251a2604abef9d904c7bc5c9b'
        Catalog = 'catalog.xml'
    }
)

foreach ($suite in $suites) {
    $suitePath = Join-Path $repositoryRoot $suite.RelativePath
    $catalogPath = Join-Path $suitePath $suite.Catalog

    if (-not (Test-Path -LiteralPath $catalogPath -PathType Leaf)) {
        throw "$($suite.Name) is not initialized at $($suite.RelativePath). Run: git submodule update --init --recursive"
    }

    $gitSafePath = $suitePath.Replace('\', '/')
    $actualRevision = & git -c "safe.directory=$gitSafePath" -C $suitePath rev-parse HEAD
    if ($LASTEXITCODE -ne 0) {
        throw "Could not read the revision for $($suite.Name)."
    }

    if ($actualRevision.Trim() -ne $suite.Revision) {
        throw "$($suite.Name) is at $($actualRevision.Trim()); expected $($suite.Revision)."
    }

    $changes = & git -c "safe.directory=$gitSafePath" -C $suitePath status --porcelain
    if ($LASTEXITCODE -ne 0) {
        throw "Could not inspect the worktree for $($suite.Name)."
    }

    if ($changes) {
        throw "$($suite.Name) contains local changes. Keep upstream suite content immutable and place FastXSLT overlays outside the submodule."
    }

    if (-not $Quiet) {
        Write-Host "$($suite.Name): $($suite.Revision)"
    }
}

if (-not $Quiet) {
    Write-Host 'Pinned conformance sources are initialized and clean.'
}
