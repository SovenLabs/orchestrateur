# Installe ou réinstalle la tâche planifiée Windows OrchestrateurDaemon.
# Usage :
#   .\scripts\install-daemon-task.ps1
#   .\scripts\install-daemon-task.ps1 -Workspace C:\GitDev\Projet\orchestrateur\workspace -StartNow

param(
    [string]$Workspace = "",
    [switch]$StartNow
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "lib\post-install.ps1")

if ([string]::IsNullOrWhiteSpace($Workspace)) {
    $Workspace = Get-OrchestrateurUserWorkspace
}

$cli = Resolve-OrchestrateurCliExe -PreferredRoot $Root
if (-not $cli) {
    throw "orch introuvable — lancez d'abord .\install.ps1 ou .\install.ps1 -Dev"
}

Initialize-OrchestrateurUserWorkspace | Out-Null
Install-OrchestrateurDaemonScheduledTask -CliExe $cli -Workspace $Workspace
if ($StartNow) {
    Start-OrchestrateurDaemonNow -CliExe $cli -Workspace $Workspace
}