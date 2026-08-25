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

Invoke-Gate 'Formatting' { cargo fmt --all --check }
Invoke-Gate 'Clippy' { cargo clippy --workspace --all-targets --all-features -- -D warnings }
Invoke-Gate 'Tests' { cargo test --workspace --all-features }
Invoke-Gate 'Markdown links' { & "$PSScriptRoot/check-markdown-links.ps1" }
Invoke-Gate 'Conformance sources' { & "$PSScriptRoot/check-conformance-sources.ps1" }

$previousRustdocFlags = $env:RUSTDOCFLAGS
try {
    $env:RUSTDOCFLAGS = '-D warnings'
    Invoke-Gate 'Documentation' { cargo doc --workspace --no-deps }
}
finally {
    $env:RUSTDOCFLAGS = $previousRustdocFlags
}

Write-Host 'All FastXSLT verification gates passed.'
