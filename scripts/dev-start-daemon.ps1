# Démarre le daemon Orchestrateur (laisser ce terminal ouvert)
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

. (Join-Path $PSScriptRoot "lib\cli.ps1")
Initialize-OrchestrateurBuildEnv
$env:ORCHESTRATEUR_DAEMON_TOKEN = "dev"

$exe = Get-OrchestrateurCliExe -Root $Root -Profile release
if (-not (Test-Path $exe)) {
    Write-Host "Binaire absent — lancement du build (1ère fois)..."
    & (Join-Path $PSScriptRoot "dev-bootstrap.ps1")
    exit $LASTEXITCODE
}

Write-Host "Daemon : http://127.0.0.1:28790/health"
& $exe daemon run --workspace workspace