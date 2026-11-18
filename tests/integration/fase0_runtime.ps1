$ErrorActionPreference = "Stop"

Write-Host "Running native Windows DFIR runtime integration test..."
cargo build --locked --workspace --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$env:DFIR_ENGINE_PATH = (Resolve-Path "target\release\dfir-engine.exe").Path
$env:DFIR_DATA_ROOT = Join-Path $env:TEMP ("dfir-runtime-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $env:DFIR_DATA_ROOT | Out-Null

try {
    & "target\release\dfir-app.exe" engine-smoke
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
finally {
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $env:DFIR_DATA_ROOT
}
