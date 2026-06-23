# Staging release v2 — CLI + Desktop Tauri + Sphère Godot (Phase 26)
param(
    [string]$StagingDir = "",
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

if (-not $Version) {
    $Version = (Select-String -Path "Cargo.toml" -Pattern '^version = "(.+)"' | ForEach-Object { $_.Matches[0].Groups[1].Value })
}
if (-not $StagingDir) {
    $StagingDir = Join-Path $Root "dist\staging-v2"
}

& (Join-Path $PSScriptRoot "stage-release.ps1") -StagingDir $StagingDir -Version $Version | Out-Null

$DesktopDir = Join-Path $StagingDir "desktop"
$SphereDir = Join-Path $StagingDir "sphere"
New-Item -ItemType Directory -Path $DesktopDir -Force | Out-Null
New-Item -ItemType Directory -Path $SphereDir -Force | Out-Null

function Copy-FirstMatch {
    param([string[]]$Candidates, [string]$DestDir, [string]$Label)
    foreach ($src in $Candidates) {
        if (Test-Path -LiteralPath $src) {
            Copy-Item -LiteralPath $src -Destination $DestDir -Force
            Write-Host "  + $Label : $(Split-Path -Leaf $src)"
            return $true
        }
    }
    Write-Host "  - $Label : introuvable (build optionnel)"
    return $false
}

Write-Host "Ajout artefacts v2 (desktop + sphère)..."

$bundleRoots = @(
    (Join-Path $Root "apps\tauri-desktop\src-tauri\target\release\bundle"),
    (Join-Path $Root "target\release\bundle")
)

$script:desktopCopied = $false
foreach ($bundleRoot in $bundleRoots) {
    if (-not (Test-Path $bundleRoot)) { continue }
    Get-ChildItem -Path $bundleRoot -Recurse -Include "*.exe", "*.msi" -ErrorAction SilentlyContinue | ForEach-Object {
        Copy-Item $_.FullName $DesktopDir -Force
        Write-Host "  + Desktop : $($_.Name)"
        $script:desktopCopied = $true
    }
    if ($script:desktopCopied) { break }
}

$sphereCandidates = @(
    (Join-Path $Root "territoire-graphique\dist\sphere\OrchestrateurSphere.exe"),
    (Join-Path $Root "dist\sphere\OrchestrateurSphere.exe")
)
Copy-FirstMatch -Candidates $sphereCandidates -DestDir $SphereDir -Label "Sphère export" | Out-Null

@"
Orchestrateur v$Version — Package hybride v2 (Phase 26)

Contenu :
  orchestrateur.exe          CLI + daemon WS (:28790)
  desktop/                   Installateur Tauri (si build effectué)
  sphere/                    Boule de Pixels Vivante standalone (si export Godot)

Démarrage rapide :
  1. $env:ORCHESTRATEUR_DAEMON_TOKEN = "dev"
  2. .\orchestrateur.exe daemon run --workspace workspace
  3. Lancer l'installateur desktop OU Godot SphereDedicated

Build complet :
  just release-v26

Documentation : docs/Phase_26_Polish_Release.md
"@ | Set-Content -Path (Join-Path $StagingDir "QUICKSTART-v2.txt") -Encoding UTF8

Write-Host "Staging v2 terminé : $StagingDir"