# Bootstrap dev — build + start daemon (Windows)
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

. (Join-Path $PSScriptRoot "lib\cli.ps1")
Initialize-OrchestrateurBuildEnv

$config = Join-Path $Root "workspace\config\orchestrator.toml"
if (-not (Test-Path $config)) {
    Copy-Item (Join-Path $Root "workspace\config\orchestrator.toml.example") $config
    Write-Host "Config créée : $config"
}

if (-not (Test-OrchestrateurCliBuilt -Root $Root -Profile release) {
    Write-Host "Compilation release (première fois, plusieurs minutes)..."
    Build-OrchestrateurCli -Root $Root -Profile release
}

$exe = Get-OrchestrateurCliExe -Root $Root -Profile release
$env:ORCHESTRATEUR_DAEMON_TOKEN = "dev"
Write-Host "Démarrage daemon sur http://127.0.0.1:28790/health"
& $exe daemon run --workspace workspace