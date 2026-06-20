# Package Windows release — Orchestrateur v0.5.0
# Usage: .\scripts\package-windows.ps1 [-OutputDir dist]

param(
    [string]$OutputDir = "dist"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$Version = (Select-String -Path "Cargo.toml" -Pattern '^version = "(.+)"' | ForEach-Object { $_.Matches[0].Groups[1].Value })
$PackageName = "Orchestrateur-v$Version-win64"
$TargetDir = Join-Path $OutputDir $PackageName

Write-Host "Build release (CLI + HUD)..."
cargo build --release -p orchestrateur-cli --features "tui,http" -p orchestrateur-hud
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "Assemblage $TargetDir ..."
if (Test-Path $TargetDir) { Remove-Item -Recurse -Force $TargetDir }
New-Item -ItemType Directory -Path $TargetDir | Out-Null
New-Item -ItemType Directory -Path (Join-Path $TargetDir "workspace") | Out-Null

Copy-Item "target\release\orchestrateur.exe" $TargetDir
Copy-Item "target\release\orchestrateur-hud.exe" $TargetDir
Copy-Item "workspace\config\orchestrator.toml.example" (Join-Path $TargetDir "workspace\orchestrator.toml.example")
Copy-Item "README.md" $TargetDir

@"
Orchestrateur v$Version — package Windows

Binaires:
  orchestrateur.exe      CLI + TUI (terminal interactif)
  orchestrateur-hud.exe  Interface graphique egui

Démarrage rapide:
  1. Copier workspace\orchestrator.toml.example vers workspace\orchestrator.toml
  2. Configurer les providers (Ollama / xAI) dans orchestrator.toml
  3. Lancer: .\orchestrateur.exe --workspace workspace
     ou:    .\orchestrateur-hud.exe

Daemon HTTP (optionnel):
  `$env:ORCHESTRATEUR_DAEMON_TOKEN = "secret"
  .\orchestrateur.exe serve --workspace workspace

Skills:
  .\orchestrateur.exe skill list --workspace workspace
  .\orchestrateur.exe skill run search --query "rust" --workspace workspace
"@ | Set-Content -Path (Join-Path $TargetDir "INSTALL.txt") -Encoding UTF8

$ZipPath = Join-Path $OutputDir "$PackageName.zip"
if (Test-Path $ZipPath) { Remove-Item -Force $ZipPath }
Compress-Archive -Path $TargetDir -DestinationPath $ZipPath

Write-Host "OK: $ZipPath"