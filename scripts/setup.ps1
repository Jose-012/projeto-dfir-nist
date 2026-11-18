$ErrorActionPreference = 'Stop'

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$InstallRoot = if ($env:DFIR_INSTALL_ROOT) { $env:DFIR_INSTALL_ROOT } else { Join-Path $env:LOCALAPPDATA 'DFIR' }
$StorageRoot = if ($env:DFIR_DATA_ROOT) { $env:DFIR_DATA_ROOT } else { Join-Path $InstallRoot 'storage' }

New-Item -ItemType Directory -Force -Path $InstallRoot | Out-Null
foreach ($Name in @('db','runtime','evidence','staging','sanitized','exports','quarantine')) {
    New-Item -ItemType Directory -Force -Path (Join-Path $StorageRoot $Name) | Out-Null
}

# O engine é resolvido ao lado do dfir-app (ADR-0015); não há mais cópia dele
# para dentro do storage de dados. Aqui só se avisa se ele ainda não foi compilado.
$BuiltEngine = Join-Path $Root 'target\release\dfir-engine.exe'
if (-not (Test-Path $BuiltEngine)) {
    Write-Warning "dfir-engine.exe was not found at $BuiltEngine. Build the workspace first (cargo build --release --workspace) or set DFIR_ENGINE_PATH explicitly."
}

Write-Host "DFIR local forensic storage initialized at $StorageRoot"
Write-Host "Engine: resolved next to dfir-app.exe (release build: $BuiltEngine); override with DFIR_ENGINE_PATH."
