# Orchestrateur — point d'entree installateur (delegue vers scripts/install.ps1)
#
# One-liner recommande (style Hermes) :
#   iex (irm https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/scripts/install.ps1)
#
# Compatibilite racine :
#   iex (irm https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.ps1)
#
# Dev (depuis clone) :
#   .\install.ps1 -Dev
#   .\install.ps1 -Dev -InstallDaemon
#
# Desinstallation :
#   irm https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/uninstall.ps1 | iex

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
    [switch]$AllUsers,
    [switch]$NonInteractive,
    [string]$Branch = "main",
    [string]$Commit = "",
    [string]$Tag = "",
    [string]$Stage = "",
    [switch]$Manifest,
    [switch]$ProtocolVersion,
    [switch]$Json
)

$ErrorActionPreference = "Stop"
$Repo = "SovenLabs/orchestrateur"
$RawBase = "https://raw.githubusercontent.com/$Repo/main"

if ($env:ORCHESTRATEUR_VERSION) { $Version = $env:ORCHESTRATEUR_VERSION }
if ($env:ORCHESTRATEUR_SILENT -eq "1") { $NonInteractive = $true }
if ($env:ORCHESTRATEUR_DEV -eq "1") { $Dev = $true }
if ($env:ORCHESTRATEUR_INSTALL_DAEMON -eq "1") { $InstallDaemon = $true }
if ($env:ORCHESTRATEUR_SKIP_DOCTOR -eq "1") { $SkipDoctor = $true }
if ($env:ORCHESTRATEUR_START_DAEMON -eq "1") { $StartDaemon = $true }
if ($env:ORCHESTRATEUR_ALL_USERS -eq "1") { $AllUsers = $true }

$localInstaller = $null
if ($PSScriptRoot) {
    $candidate = Join-Path $PSScriptRoot "scripts\install.ps1"
    if (Test-Path -LiteralPath $candidate) {
        $localInstaller = $candidate
    }
}

if (-not $localInstaller) {
    $localInstaller = Join-Path $env:TEMP "orchestrateur-scripts-install.ps1"
    Write-Host "Telechargement de l'installateur complet..."
    Invoke-WebRequest -Uri "$RawBase/scripts/install.ps1" -OutFile $localInstaller -UseBasicParsing
}

$invokeArgs = @{
    Dev            = $Dev
    Debug          = $Debug
    SkipBuild      = $SkipBuild
    InstallDaemon  = $InstallDaemon
    SkipDoctor     = $SkipDoctor
    StartDaemon    = $StartDaemon
    AllUsers       = $AllUsers
    NonInteractive = $NonInteractive
    Manifest       = $Manifest
    ProtocolVersion = $ProtocolVersion
    Json           = $Json
    Branch         = $Branch
    Commit         = $Commit
    Tag            = $Tag
}
if ($Version) { $invokeArgs.Version = $Version }
if ($Stage) { $invokeArgs.Stage = $Stage }

& $localInstaller @invokeArgs
exit $LASTEXITCODE