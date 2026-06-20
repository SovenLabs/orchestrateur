# Orchestrateur — installation Windows (telecharge le Setup.exe depuis GitHub Releases)
# Usage direct:
#   irm https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.ps1 | iex
# Version fixe:
#   $env:ORCHESTRATEUR_VERSION = "0.5.0"; irm ... | iex
# Silencieux:
#   $env:ORCHESTRATEUR_SILENT = "1"; irm ... | iex

#Requires -Version 5.1

param(
    [string]$Version = "",
    [switch]$Silent
)

$ErrorActionPreference = "Stop"

$Repo = "SovenLabs/orchestrateur"
if ($env:ORCHESTRATEUR_VERSION) {
    $Version = $env:ORCHESTRATEUR_VERSION
}
if ($env:ORCHESTRATEUR_SILENT -eq "1") {
    $Silent = $true
}

function Get-LatestReleaseVersion {
    $headers = @{ "User-Agent" = "Orchestrateur-Install-Script" }
    $api = "https://api.github.com/repos/$Repo/releases/latest"
    $release = Invoke-RestMethod -Uri $api -Headers $headers -UseBasicParsing
    $tag = [string]$release.tag_name
    if ($tag.StartsWith("v")) {
        return $tag.Substring(1)
    }
    return $tag
}

function Normalize-Version([string]$Value) {
    $v = $Value.Trim()
    if ($v.StartsWith("v")) {
        $v = $v.Substring(1)
    }
    return $v
}

if ([string]::IsNullOrWhiteSpace($Version)) {
    Write-Host "Recherche de la derniere release GitHub..."
    $Version = Get-LatestReleaseVersion
}

$Version = Normalize-Version $Version
$AssetName = "Orchestrateur-v$Version-Setup-win64.exe"
$DownloadUrl = "https://github.com/$Repo/releases/download/v$Version/$AssetName"
$TempDir = Join-Path $env:TEMP "orchestrateur-install"
$SetupPath = Join-Path $TempDir $AssetName

if (-not (Test-Path $TempDir)) {
    New-Item -ItemType Directory -Path $TempDir | Out-Null
}

Write-Host "Orchestrateur v$Version"
Write-Host "Telechargement: $DownloadUrl"
try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $SetupPath -UseBasicParsing
} catch {
    throw @"
Echec du telechargement ($AssetName).
Publiez d'abord une release GitHub avec le Setup.exe:
  .\scripts\publish-github-release.ps1
Ou installez depuis le depot: https://github.com/$Repo/releases
"@
}

if (-not (Test-Path $SetupPath)) {
    throw "Fichier installeur introuvable apres telechargement: $SetupPath"
}

Write-Host "Lancement de l'installeur..."
$args = if ($Silent) { "/SILENT", "/SUPPRESSMSGBOXES", "/NORESTART" } else { @() }
$proc = Start-Process -FilePath $SetupPath -ArgumentList $args -PassThru -Wait
if ($proc.ExitCode -ne 0) {
    throw "Installeur termine avec le code $($proc.ExitCode)"
}

Write-Host "Installation terminee."
Write-Host "Workspace utilisateur: $env:APPDATA\Orchestrateur\workspace"
Write-Host "Lancez « Orchestrateur » depuis le menu Demarrer."