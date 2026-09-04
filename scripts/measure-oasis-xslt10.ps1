param(
    [string]$ArchivePath = (Join-Path $PSScriptRoot '..\.workbench\oasis-xslt10\XSLT-testsuite-04.ZIP'),
    [string]$ExtractedTestsPath = (Join-Path $PSScriptRoot '..\.workbench\oasis-xslt10\extracted-full\testsuite\TESTS')
)

$ErrorActionPreference = 'Stop'
$expectedHash = '66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5'
$resolvedArchive = (Resolve-Path -LiteralPath $ArchivePath).Path
$resolvedTests = (Resolve-Path -LiteralPath $ExtractedTestsPath).Path
$actualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $resolvedArchive).Hash
if ($actualHash -ne $expectedHash) {
    throw "OASIS XSLT 1.0 archive hash mismatch: expected $expectedHash, found $actualHash"
}

$catalogPath = Join-Path $resolvedTests 'catalog.xml'
$doubtsPath = Join-Path $resolvedTests 'doubts.xml'
if (-not (Test-Path -LiteralPath $catalogPath -PathType Leaf) -or
    -not (Test-Path -LiteralPath $doubtsPath -PathType Leaf)) {
    throw 'Extracted OASIS TESTS directory must contain catalog.xml and doubts.xml'
}

[xml]$catalog = Get-Content -Raw -LiteralPath $catalogPath
$cases = @($catalog.'test-suite'.'test-catalog'.'test-case')
if ($cases.Count -ne 3173) {
    throw "OASIS XSLT 1.0 catalog denominator changed: expected 3173, found $($cases.Count)"
}

$priorRoot = $env:FASTXSLT_OASIS_XSLT10_ROOT
try {
    $env:FASTXSLT_OASIS_XSLT10_ROOT = $resolvedTests
    & cargo test --release -p fastxslt --all-features measures_local_oasis_xslt10_compatibility -- --ignored --nocapture
    if ($LASTEXITCODE -ne 0) {
        throw "OASIS XSLT 1.0 compatibility measurement failed with exit code $LASTEXITCODE"
    }
}
finally {
    $env:FASTXSLT_OASIS_XSLT10_ROOT = $priorRoot
}
