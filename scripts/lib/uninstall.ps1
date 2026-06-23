# Désinstallation complète Orchestrateur — stop sécurité (processus, PATH, binaires, tâche).
$script:OrchestrateurDaemonTaskName = "OrchestrateurDaemon"
$script:OrchestrateurDaemonTokenEnvName = "ORCHESTRATEUR_DAEMON_TOKEN"

function Stop-OrchestrateurProcesses {
    $names = @("orch.exe", "orchestrateur.exe", "orchestre.exe")
    foreach ($name in $names) {
        $null = Start-Process -FilePath "taskkill" -ArgumentList @("/F", "/IM", $name) `
            -Wait -NoNewWindow -WindowStyle Hidden -RedirectStandardError "$env:TEMP\orch-uninstall-null.err" `
            -RedirectStandardOutput "$env:TEMP\orch-uninstall-null.out" -ErrorAction SilentlyContinue
    }
    Write-Host "Processus Orchestrateur arretes (ou deja absents)."
}

function Remove-OrchestrateurScheduledTask {
    $taskName = $script:OrchestrateurDaemonTaskName
    $null = Start-Process -FilePath "schtasks.exe" -ArgumentList @("/End", "/TN", $taskName) `
        -Wait -NoNewWindow -WindowStyle Hidden -RedirectStandardError "$env:TEMP\orch-uninstall-null.err" `
        -RedirectStandardOutput "$env:TEMP\orch-uninstall-null.out" -ErrorAction SilentlyContinue
    try {
        $existing = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
        if ($existing) {
            Unregister-ScheduledTask -TaskName $taskName -Confirm:$false
            Write-Host "Tache planifiee supprimee : $taskName"
        } else {
            Write-Host "Tache planifiee absente : $taskName"
        }
    } catch {
        Write-Warning "Impossible de supprimer la tache $taskName : $($_.Exception.Message)"
    }
}

function Remove-OrchestrateurPathEntry {
    param(
        [Parameter(Mandatory = $true)][string]$Dir,
        [ValidateSet("User", "Machine")][string]$Scope = "User"
    )
    $current = [Environment]::GetEnvironmentVariable("Path", $Scope)
    if ([string]::IsNullOrWhiteSpace($current)) {
        return
    }
    $normalizedDir = $Dir.TrimEnd('\')
    $parts = $current -split ';' | Where-Object {
        $_ -and ($_.TrimEnd('\') -ne $normalizedDir)
    }
    $newPath = ($parts | Select-Object -Unique) -join ';'
    if ($newPath -eq $current) {
        Write-Host "PATH $Scope : $Dir deja absent"
        return
    }
    [Environment]::SetEnvironmentVariable("Path", $newPath, $Scope)
    Write-Host "PATH $Scope : $Dir retire"
}

function Remove-OrchestrateurDirectory {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [string]$Label = ""
    )
    if (-not (Test-Path -LiteralPath $Path)) {
        if ($Label) { Write-Host "$Label : deja absent ($Path)" }
        return
    }
    try {
        Remove-Item -LiteralPath $Path -Recurse -Force -ErrorAction Stop
        Write-Host "Supprime : $Path"
    } catch {
        Write-Warning "Impossible de supprimer $Path (fichier verrouille ?) : $($_.Exception.Message)"
    }
}

function Clear-OrchestrateurDaemonToken {
    $name = $script:OrchestrateurDaemonTokenEnvName
    [Environment]::SetEnvironmentVariable($name, $null, "User")
    Remove-Item -Path "env:$name" -ErrorAction SilentlyContinue
    Write-Host "Variable utilisateur retiree : $name"
}

function Uninstall-OrchestrateurCliArtifacts {
    param([switch]$AllUsers)

    $userBin = Get-OrchestrateurUserBinDir
    Remove-OrchestrateurDirectory -Path $userBin -Label "bin utilisateur"
    Remove-OrchestrateurCliFromLegacyDirs

    if ($AllUsers) {
        if (-not (Test-OrchestrateurIsAdministrator)) {
            throw @"
Retrait systeme (-AllUsers) : lancez PowerShell en administrateur.
  Exemple : .\uninstall.ps1 -AllUsers
"@
        }
        $machineDir = Join-Path ${env:ProgramFiles} "Orchestrateur"
        Remove-OrchestrateurDirectory -Path $machineDir -Label "bin systeme"
    }
}

function Uninstall-OrchestrateurPathEntries {
    param([switch]$AllUsers)

    Remove-OrchestrateurPathEntry -Dir (Get-OrchestrateurUserBinDir) -Scope "User"
    if ($AllUsers) {
        Remove-OrchestrateurPathEntry -Dir (Join-Path ${env:ProgramFiles} "Orchestrateur") -Scope "Machine"
    }
}

function Uninstall-OrchestrateurComplete {
    param(
        [switch]$PurgeData,
        [switch]$AllUsers
    )

    Write-Host "=== Desinstallation Orchestrateur (stop securite) ===" -ForegroundColor Cyan

    Stop-OrchestrateurProcesses
    Remove-OrchestrateurScheduledTask
    Uninstall-OrchestrateurCliArtifacts -AllUsers:$AllUsers
    Uninstall-OrchestrateurPathEntries -AllUsers:$AllUsers
    Clear-OrchestrateurDaemonToken

    if ($PurgeData) {
        $dataRoot = Join-Path $env:APPDATA "Orchestrateur"
        Remove-OrchestrateurDirectory -Path $dataRoot -Label "donnees utilisateur"
    } else {
        Write-Host "Donnees conservees : $(Join-Path $env:APPDATA 'Orchestrateur') (utilisez -PurgeData pour effacer)"
    }

    Write-Host ""
    Write-Host "Desinstallation terminee." -ForegroundColor Green
    Write-Host "Fermez et rouvrez le terminal - orchestrateur ne doit plus etre reconnu."
}