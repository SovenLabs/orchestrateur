# Publie une release GitHub (Setup.exe + zip + checksums + install scripts)
# Prerequis: gh auth login, artefacts dans dist/
# Usage:
#   .\scripts\build-installer.ps1 -InstallInno
#   .\scripts\publish-github-release.ps1
#   .\scripts\publish-github-release.ps1 -Version 0.5.0 -Draft

param(
    [string]$Version = "",
    [switch]$Draft,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

if (-not $Version) {
    $Version = (Select-String -Path "Cargo.toml" -Pattern '^version = "(.+)"' | ForEach-Object { $_.Matches[0].Groups[1].Value })
}

$Tag = "v$Version"
$DistDir = Join-Path $Root "dist"
$SetupExe = Join-Path $DistDir "Orchestrateur-v$Version-Setup-win64.exe"
$ZipFile = Join-Path $DistDir "Orchestrateur-v$Version-win64.zip"

if (-not $SkipBuild) {
    & (Join-Path $PSScriptRoot "build-installer.ps1") -InstallInno
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

if (-not (Test-Path $SetupExe)) {
    throw "Setup manquant: $SetupExe - lancez build-installer.ps1 avant publication"
}
if (-not (Test-Path $ZipFile)) {
    throw "Zip manquant: $ZipFile"
}

$gh = Get-Command gh -ErrorAction SilentlyContinue
if (-not $gh) {
    throw "GitHub CLI (gh) requis: https://cli.github.com/"
}

$notes = @"
Orchestrateur v$Version — Windows x64

## Installation rapide

**PowerShell (recommande):**
``````powershell
irm https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.ps1 | iex
``````

**curl (Git Bash / WSL):**
``````bash
curl -fsSL https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.sh | sh
``````

Version fixe: ``ORCHESTRATEUR_VERSION=$Version`` avant la commande.

## Fichiers

- ``Orchestrateur-v$Version-Setup-win64.exe`` — installateur Inno Setup
- ``Orchestrateur-v$Version-win64.zip`` — archive portable

Workspace utilisateur: ``%APPDATA%\Orchestrateur\workspace``
"@

$NotesPath = Join-Path $DistDir "RELEASE_NOTES_v$Version.md"
$notes | Set-Content -Path $NotesPath -Encoding UTF8

# Checksums pour verification optionnelle
$ShaPath = Join-Path $DistDir "SHA256SUMS.txt"
$setupHash = (Get-FileHash $SetupExe -Algorithm SHA256).Hash.ToLower()
$zipHash = (Get-FileHash $ZipFile -Algorithm SHA256).Hash.ToLower()
@"
$setupHash  $(Split-Path $SetupExe -Leaf)
$zipHash  $(Split-Path $ZipFile -Leaf)
"@ | Set-Content -Path $ShaPath -Encoding ASCII

$draftFlag = if ($Draft) { "--draft" } else { "" }

Write-Host "Publication release $Tag..."
$releaseExists = $false
try {
    gh release view $Tag 2>$null | Out-Null
    $releaseExists = ($LASTEXITCODE -eq 0)
} catch {
    $releaseExists = $false
}
if ($releaseExists) {
    gh release upload $Tag $SetupExe $ZipFile $ShaPath --clobber
    Write-Host "Assets mis a jour sur la release existante $Tag"
} else {
    if ($draftFlag) {
        gh release create $Tag $SetupExe $ZipFile $ShaPath --title "Orchestrateur v$Version" --notes-file $NotesPath --draft
    } else {
        gh release create $Tag $SetupExe $ZipFile $ShaPath --title "Orchestrateur v$Version" --notes-file $NotesPath
    }
    Write-Host "Release creee: $Tag"
}

Write-Host "URL: https://github.com/SovenLabs/orchestrateur/releases/tag/$Tag"