# Orchestrateur - desinstallation Windows (stop securite)
#
# Depuis le depot :
#   powershell -ExecutionPolicy Bypass -File .\uninstall.ps1
#   powershell -ExecutionPolicy Bypass -File .\uninstall.ps1 -PurgeData
#   powershell -ExecutionPolicy Bypass -File .\uninstall.ps1 -AllUsers
#
# Depuis GitHub :
#   irm https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/uninstall.ps1 | iex
#   $env:ORCHESTRATEUR_PURGE_DATA = "1"; irm ... | iex
#   $env:ORCHESTRATEUR_ALL_USERS = "1"; irm ... | iex

#Requires -Version 5.1

param(
    [switch]$PurgeData,
    [switch]$AllUsers
)

$ErrorActionPreference = "Stop"
$Repo = "SovenLabs/orchestrateur"
$RawBase = "https://raw.githubusercontent.com/$Repo/main"
$OrchestrateurRoot = $PSScriptRoot
if (-not $OrchestrateurRoot) {
    $OrchestrateurRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
}

$localCli = Join-Path $OrchestrateurRoot "scripts\lib\cli.ps1"
$localUninstall = Join-Path $OrchestrateurRoot "scripts\lib\uninstall.ps1"
if ((Test-Path -LiteralPath $localCli) -and (Test-Path -LiteralPath $localUninstall)) {
    . $localCli
    . $localUninstall
} else {
    $libDir = Join-Path $env:TEMP "orchestrateur-uninstall-lib"
    if (-not (Test-Path $libDir)) {
        New-Item -ItemType Directory -Path $libDir -Force | Out-Null
    }
    $files = @(
        @{ Name = "cli.ps1"; Url = "$RawBase/scripts/lib/cli.ps1" },
        @{ Name = "uninstall.ps1"; Url = "$RawBase/scripts/lib/uninstall.ps1" }
    )
    foreach ($f in $files) {
        $dest = Join-Path $libDir $f.Name
        Invoke-WebRequest -Uri $f.Url -OutFile $dest -UseBasicParsing
    }
    . (Join-Path $libDir "cli.ps1")
    . (Join-Path $libDir "uninstall.ps1")
}

if ($env:ORCHESTRATEUR_PURGE_DATA -eq "1") { $PurgeData = $true }
if ($env:ORCHESTRATEUR_ALL_USERS -eq "1") { $AllUsers = $true }

Uninstall-OrchestrateurComplete -PurgeData:$PurgeData -AllUsers:$AllUsers