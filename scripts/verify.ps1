[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

function Invoke-Gate {
    param(
        [Parameter(Mandatory)]
        [string]$Name,

        [Parameter(Mandatory)]
        [scriptblock]$Command
    )

    Write-Host "==> $Name"
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE"
    }
}

Write-Host '==> Unsafe surface'
$nativeBoundary = Join-Path $PSScriptRoot '../crates/fastxslt-dotnet-workbench/src/lib.rs'
$nativeSource = Get-Content -LiteralPath $nativeBoundary -Raw
$unsafeBlocks = [regex]::Matches($nativeSource, '\bunsafe\s*\{').Count
$unsafeExports = [regex]::Matches($nativeSource, '#\[unsafe\(no_mangle\)\]').Count
$unsafeAllowances = [regex]::Matches($nativeSource, '#\[allow\(unsafe_code').Count
if ($unsafeBlocks -ne 2 -or $unsafeExports -ne 16 -or $unsafeAllowances -ne 18) {
    throw "ADR-0008 through ADR-0011 unsafe surface changed: blocks=$unsafeBlocks exports=$unsafeExports allowances=$unsafeAllowances"
}
$otherUnsafe = Get-ChildItem (Join-Path $PSScriptRoot '../crates') -Recurse -Filter '*.rs' |
    Where-Object FullName -ne (Resolve-Path -LiteralPath $nativeBoundary).Path |
    Select-String -Pattern '\bunsafe\s*(\{|fn\b|trait\b|impl\b)|#\[unsafe\('
if ($otherUnsafe) {
    throw "Unsafe Rust appeared outside the ADR-0008 native boundary: $($otherUnsafe.Path -join ', ')"
}

Invoke-Gate 'Formatting' { cargo fmt --all --check }
Invoke-Gate 'Clippy' { cargo clippy --workspace --all-targets --all-features -- -D warnings }
Invoke-Gate 'Tests' { cargo test --workspace --all-features }
Invoke-Gate 'Markdown links' { & "$PSScriptRoot/check-markdown-links.ps1" }
Invoke-Gate 'Conformance sources' { & "$PSScriptRoot/check-conformance-sources.ps1" }
Invoke-Gate 'Conformance inventory' { & "$PSScriptRoot/inventory-conformance-sources.ps1" }
Invoke-Gate 'XSLT30 metadata inventory' {
    & "$PSScriptRoot/inventory-xslt30-case-metadata.ps1" | Out-Null
}

$previousRustdocFlags = $env:RUSTDOCFLAGS
try {
    $env:RUSTDOCFLAGS = '-D warnings'
    Invoke-Gate 'Documentation' { cargo doc --workspace --no-deps }
}
finally {
    $env:RUSTDOCFLAGS = $previousRustdocFlags
}

Write-Host 'All FastXSLT verification gates passed.'
