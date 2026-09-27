param(
    [ValidateRange(1, 1000000)]
    [int]$Iterations = 2000
)

$ErrorActionPreference = 'Stop'
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$outputDirectory = Join-Path $repositoryRoot '.workbench/wasm-bindgen'
$wasmPath = Join-Path $repositoryRoot 'target/wasm32-unknown-unknown/release/fastxslt_wasm_workbench.wasm'
$benchmarkPath = Join-Path $repositoryRoot 'workbenches/FastXSLT.Wasm.Workbench/benchmark.mjs'

foreach ($command in @('cargo', 'wasm-bindgen', 'node')) {
    if (-not (Get-Command $command -ErrorAction SilentlyContinue)) {
        throw "Required WASM workbench command '$command' is not available on PATH."
    }
}

Push-Location $repositoryRoot
try {
    cargo build --release -p fastxslt-wasm-workbench --target wasm32-unknown-unknown
    if ($LASTEXITCODE -ne 0) {
        throw "WASM workbench build failed with exit code $LASTEXITCODE"
    }

    New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null
    wasm-bindgen $wasmPath --target nodejs --out-dir $outputDirectory --no-typescript
    if ($LASTEXITCODE -ne 0) {
        throw "wasm-bindgen failed with exit code $LASTEXITCODE"
    }

    node $benchmarkPath $Iterations
    if ($LASTEXITCODE -ne 0) {
        throw "WASM benchmark failed with exit code $LASTEXITCODE"
    }
}
finally {
    Pop-Location
}
