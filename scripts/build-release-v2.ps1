# Build release v2 — Rust CLI + Tauri desktop + staging zip (Phase 26)
# Usage:
#   .\scripts\build-release-v2.ps1
#   .\scripts\build-release-v2.ps1 -SkipTauri -SkipSphere
#   .\scripts\build-release-v2.ps1 -ZipOnly

param(
    [switch]$SkipBuild,
    [switch]$SkipTauri,
    [switch]$SkipSphere,
    [switch]$ZipOnly
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$Version = (Select-String -Path "Cargo.toml" -Pattern '^version = "(.+)"' | ForEach-Object { $_.Matches[0].Groups[1].Value })
$DistDir = Join-Path $Root "dist"
$StagingDir = Join-Path $DistDir "staging-v2"
$PackageName = "Orchestrateur-v$Version-hybrid-win64"

if (-not $SkipBuild) {
    Write-Host "== Export types TypeScript =="
    cargo run -p shared-types --bin export-ts
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    Write-Host "== Compilation release Rust (CLI + daemon) =="
    . (Join-Path $PSScriptRoot "lib\cli.ps1")
    Initialize-OrchestrateurBuildEnv
    Build-OrchestrateurCli -Root $Root -Profile release -ExtraFeatures http

    if (-not $SkipTauri) {
        Write-Host "== Build Tauri desktop =="
        Push-Location (Join-Path $Root "apps\tauri-desktop")
        if (-not (Test-Path "node_modules")) {
            npm install
            if ($LASTEXITCODE -ne 0) { Pop-Location; exit $LASTEXITCODE }
        }
        npm run tauri build
        $tauriExit = $LASTEXITCODE
        Pop-Location
        if ($tauriExit -ne 0) { exit $tauriExit }
    }

    if (-not $SkipSphere) {
        Write-Host "== Export sphère Godot (optionnel) =="
        $sphereScript = Join-Path $Root "territoire-graphique\scripts\export-sphere.ps1"
        if (Test-Path $sphereScript) {
            & powershell -NoProfile -ExecutionPolicy Bypass -File $sphereScript
            if ($LASTEXITCODE -ne 0) {
                Write-Host "Export sphère ignoré (Godot absent ou erreur export)."
            }
        }
    }
}

& (Join-Path $PSScriptRoot "stage-release-v2.ps1") -StagingDir $StagingDir -Version $Version | Out-Null

$ZipPath = Join-Path $DistDir "$PackageName.zip"
if (Test-Path $ZipPath) { Remove-Item -Force $ZipPath }
Compress-Archive -Path $StagingDir -DestinationPath $ZipPath -Force
Write-Host ""
Write-Host "ZIP hybride v2 : $ZipPath"

if ($ZipOnly) { exit 0 }

Write-Host ""
Write-Host "Validation recommandée :"
Write-Host "  just test-ws"
Write-Host "  just desktop-test"
Write-Host "  Tag : .\scripts\tag-phase-release.ps1 -Phase 26 -Version $Version"