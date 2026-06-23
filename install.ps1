# Orchestrateur — installateur Windows unique
#
# Desinstallation : powershell -ExecutionPolicy Bypass -File .\uninstall.ps1
#   irm https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/uninstall.ps1 | iex
#
# Release (Setup.exe depuis GitHub) :
#   irm https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.ps1 | iex
#   $env:ORCHESTRATEUR_VERSION = "0.28.0"; irm ... | iex
#   $env:ORCHESTRATEUR_SILENT = "1"; irm ... | iex
#   irm ... | iex; install.ps1 -InstallDaemon
#
# Dev (compile depuis le dépôt, installe orch.exe dans PATH) :
#   .\install.ps1 -Dev
#   .\install.ps1 -Dev -InstallDaemon
#   .\install.ps1 -Dev -AllUsers          # PATH systeme (CMD admin)
#   $env:ORCHESTRATEUR_DEV = "1"; .\install.ps1

#Requires -Version 5.1

param(
    [string]$Version = "",
    [switch]$Silent,
    [switch]$Dev,
    [switch]$Debug,
    [switch]$SkipBuild,
    [switch]$InstallDaemon,
    [switch]$SkipDoctor,
    [switch]$StartDaemon,
    [switch]$AllUsers
)

$ErrorActionPreference = "Stop"
$Repo = "SovenLabs/orchestrateur"
$RawBase = "https://raw.githubusercontent.com/$Repo/main"

function Import-OrchestrateurInstallLibs {
    $localPost = if ($PSScriptRoot) { Join-Path $PSScriptRoot "scripts\lib\post-install.ps1" } else { "" }
    if ($localPost -and (Test-Path -LiteralPath $localPost)) {
        . $localPost
        return
    }
    $libDir = Join-Path $env:TEMP "orchestrateur-install-lib"
    if (-not (Test-Path $libDir)) {
        New-Item -ItemType Directory -Path $libDir -Force | Out-Null
    }
    $files = @(
        @{ Name = "cli.ps1"; Url = "$RawBase/scripts/lib/cli.ps1" },
        @{ Name = "post-install.ps1"; Url = "$RawBase/scripts/lib/post-install.ps1" }
    )
    foreach ($f in $files) {
        $dest = Join-Path $libDir $f.Name
        Invoke-WebRequest -Uri $f.Url -OutFile $dest -UseBasicParsing
    }
    . (Join-Path $libDir "post-install.ps1")
}

if ($env:ORCHESTRATEUR_VERSION) { $Version = $env:ORCHESTRATEUR_VERSION }
if ($env:ORCHESTRATEUR_SILENT -eq "1") { $Silent = $true }
if ($env:ORCHESTRATEUR_DEV -eq "1") { $Dev = $true }
if ($env:ORCHESTRATEUR_INSTALL_DAEMON -eq "1") { $InstallDaemon = $true }
if ($env:ORCHESTRATEUR_SKIP_DOCTOR -eq "1") { $SkipDoctor = $true }
if ($env:ORCHESTRATEUR_START_DAEMON -eq "1") { $StartDaemon = $true }
if ($env:ORCHESTRATEUR_ALL_USERS -eq "1") { $AllUsers = $true }

if ($Dev) {
    $Root = $PSScriptRoot
    if (-not (Test-Path (Join-Path $Root "Cargo.toml"))) {
        throw "Mode -Dev : exécutez install.ps1 depuis la racine du dépôt orchestrateur."
    }
    Set-Location $Root
    . (Join-Path $Root "scripts\lib\cli.ps1")
    Initialize-OrchestrateurBuildEnv

    $profile = if ($Debug) { "debug" } else { "release" }
    if (-not $SkipBuild) {
        Build-OrchestrateurCli -Root $Root -Profile $profile
    } else {
        Remove-OrchestrateurLegacyCliArtifacts -Root $Root -Profile $profile
    }

    Install-OrchestrateurCliToUserPath -Root $Root -Profile $profile
    if ($AllUsers) {
        Install-OrchestrateurCliToMachinePath -Root $Root -Profile $profile
    }
    $ws = Join-Path $Root "workspace"
    Write-Host ""
    Write-Host "Installation dev terminee."
    . (Join-Path $Root "scripts\lib\post-install.ps1")
    Complete-OrchestrateurPostInstall `
        -PreferredRoot $Root `
        -Workspace $ws `
        -InstallDaemon:$InstallDaemon `
        -SkipDoctor:$SkipDoctor `
        -StartDaemon:$StartDaemon
    Write-Host "Ouvrez un NOUVEAU terminal puis tapez : orchestrateur --version"
    exit 0
}

function Get-LatestReleaseVersion {
    $headers = @{ "User-Agent" = "Orchestrateur-Install-Script" }
    $api = "https://api.github.com/repos/$Repo/releases/latest"
    $release = Invoke-RestMethod -Uri $api -Headers $headers -UseBasicParsing
    $tag = [string]$release.tag_name
    if ($tag.StartsWith("v")) { return $tag.Substring(1) }
    return $tag
}

function Normalize-Version([string]$Value) {
    $v = $Value.Trim()
    if ($v.StartsWith("v")) { $v = $v.Substring(1) }
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
Installez depuis le depot (dev) : .\install.ps1 -Dev
Ou publiez une release : .\scripts\publish-github-release.ps1
Releases : https://github.com/$Repo/releases
"@
}

if (-not (Test-Path $SetupPath)) {
    throw "Fichier installeur introuvable apres telechargement: $SetupPath"
}

Write-Host "Lancement de l'installeur Inno Setup..."
$setupArgs = if ($Silent) { "/SILENT", "/SUPPRESSMSGBOXES", "/NORESTART" } else { @() }
$proc = Start-Process -FilePath $SetupPath -ArgumentList $setupArgs -PassThru -Wait
if ($proc.ExitCode -ne 0) {
    throw "Installeur termine avec le code $($proc.ExitCode)"
}

Write-Host ""
Write-Host "Setup.exe termine - finalisation harness..." -ForegroundColor Cyan
Import-OrchestrateurInstallLibs
Complete-OrchestrateurPostInstall `
    -InstallDaemon:$InstallDaemon `
    -SkipDoctor:$SkipDoctor `
    -StartDaemon:$StartDaemon