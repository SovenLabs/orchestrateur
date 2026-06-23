# Post-install release/dev - workspace, doctor, daemon Windows (Scheduled Task).
$script:OrchestrateurDaemonTaskName = "OrchestrateurDaemon"

function Get-OrchestrateurUserWorkspace {
    return Join-Path $env:APPDATA "Orchestrateur\workspace"
}

function Get-OrchestrateurDaemonTokenEnvName {
    return "ORCHESTRATEUR_DAEMON_TOKEN"
}

function Refresh-OrchestrateurUserPath {
    $machine = [Environment]::GetEnvironmentVariable("Path", "Machine")
    $user = [Environment]::GetEnvironmentVariable("Path", "User")
    $parts = @()
    if ($machine) { $parts += $machine -split ';' }
    if ($user) { $parts += $user -split ';' }
    $env:Path = ($parts | Where-Object { $_ } | Select-Object -Unique) -join ';'
}

function Resolve-OrchestrateurCliExe {
    param([string]$PreferredRoot = "")
    . (Join-Path $PSScriptRoot "cli.ps1")

    $candidates = @()
    if ($PreferredRoot) {
        $candidates += Join-Path $PreferredRoot "target\release\orch.exe"
    }
    $candidates += @(
        (Join-Path (Get-OrchestrateurUserBinDir) "orch.exe"),
        (Join-Path $env:USERPROFILE ".cargo\bin\orch.exe"),
        (Join-Path $env:LOCALAPPDATA "Programs\Orchestrateur\orch.exe"),
        (Join-Path $env:ProgramFiles "Orchestrateur\orch.exe"),
        (Join-Path ${env:ProgramFiles(x86)} "Orchestrateur\orch.exe")
    )
    foreach ($path in $candidates) {
        if ($path -and (Test-Path -LiteralPath $path)) { return $path }
    }
    foreach ($name in @("orchestrateur", "orch", "orchestre")) {
        $cmd = Get-Command $name -ErrorAction SilentlyContinue
        if ($cmd -and (Test-Path -LiteralPath $cmd.Source)) {
            $src = $cmd.Source
            if ($src.EndsWith(".cmd", [StringComparison]::OrdinalIgnoreCase)) {
                $orch = Join-Path (Split-Path -Parent $src) "orch.exe"
                if (Test-Path -LiteralPath $orch) { return $orch }
            }
            if ($src.EndsWith(".exe", [StringComparison]::OrdinalIgnoreCase)) { return $src }
        }
    }
    return $null
}

function Initialize-OrchestrateurUserWorkspace {
    param([string]$ExampleConfig = "")
    $root = Get-OrchestrateurUserWorkspace
    $configDir = Join-Path $root "config"
    $configFile = Join-Path $configDir "orchestrator.toml"
    $dirs = @(
        $root,
        (Join-Path $root "memories"),
        (Join-Path $root "logs"),
        $configDir
    )
    foreach ($dir in $dirs) {
        if (-not (Test-Path $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
        }
    }
    if ((Test-Path $configFile) -or [string]::IsNullOrWhiteSpace($ExampleConfig)) {
        return $configFile
    }
    if (-not (Test-Path $ExampleConfig)) {
        Write-Warning "Exemple config introuvable : $ExampleConfig"
        return $configFile
    }
    Copy-Item -LiteralPath $ExampleConfig -Destination $configFile -Force
    Write-Host "Config créée : $configFile"
    return $configFile
}

function Ensure-OrchestrateurDaemonToken {
    $name = Get-OrchestrateurDaemonTokenEnvName
    $existing = [Environment]::GetEnvironmentVariable($name, "User")
    if (-not [string]::IsNullOrWhiteSpace($existing)) {
        Set-Item -Path "env:$name" -Value $existing
        Write-Host "Token daemon : déjà défini ($name, utilisateur)"
        return $existing
    }
    $token = [guid]::NewGuid().ToString("N")
    [Environment]::SetEnvironmentVariable($name, $token, "User")
    Set-Item -Path "env:$name" -Value $token
    Write-Host "Token daemon généré et enregistré ($name, utilisateur)"
    return $token
}

function Invoke-OrchestrateurDoctor {
    param(
        [Parameter(Mandatory = $true)][string]$CliExe,
        [Parameter(Mandatory = $true)][string]$Workspace
    )
    if (-not (Test-Path -LiteralPath $CliExe)) {
        Write-Warning "doctor ignoré - CLI introuvable"
        return $false
    }
    if (-not (Test-Path -LiteralPath $Workspace)) {
        Write-Warning "doctor ignoré - workspace introuvable : $Workspace"
        return $false
    }
    Write-Host ""
    Write-Host "=== Vérification harness (orch doctor) ===" -ForegroundColor Cyan
    & $CliExe doctor --workspace $Workspace
    if ($LASTEXITCODE -ne 0) {
        Write-Warning "doctor a signalé des problèmes (code $LASTEXITCODE) - le harness peut nécessiter une config LLM optionnelle."
        return $false
    }
    Write-Host "doctor : OK" -ForegroundColor Green
    return $true
}

function Install-OrchestrateurDaemonScheduledTask {
    param(
        [Parameter(Mandatory = $true)][string]$CliExe,
        [Parameter(Mandatory = $true)][string]$Workspace
    )
    Ensure-OrchestrateurDaemonToken | Out-Null
    $taskName = $script:OrchestrateurDaemonTaskName
    $workDir = Split-Path -Parent $CliExe
    $args = "daemon run --workspace `"$Workspace`""
    try {
        $existing = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
        if ($existing) {
            Unregister-ScheduledTask -TaskName $taskName -Confirm:$false
        }
        $action = New-ScheduledTaskAction -Execute $CliExe -Argument $args -WorkingDirectory $workDir
        $trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
        $settings = New-ScheduledTaskSettingsSet `
            -AllowStartIfOnBatteries `
            -DontStopIfGoingOnBatteries `
            -StartWhenAvailable `
            -RestartCount 3 `
            -RestartInterval (New-TimeSpan -Minutes 1)
        $principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -LogonType Interactive -RunLevel Limited
        Register-ScheduledTask `
            -TaskName $taskName `
            -Action $action `
            -Trigger $trigger `
            -Settings $settings `
            -Principal $principal `
            -Description "Orchestrateur harness daemon (WS http://127.0.0.1:28790)" | Out-Null
        Write-Host "Tâche planifiée installée : $taskName (démarrage à la connexion)" -ForegroundColor Green
        return $true
    } catch {
        Write-Warning "Échec tâche planifiée : $($_.Exception.Message)"
        Write-Host "Lancez manuellement : orch daemon run --workspace `"$Workspace`""
        return $false
    }
}

function Start-OrchestrateurDaemonNow {
    param(
        [Parameter(Mandatory = $true)][string]$CliExe,
        [Parameter(Mandatory = $true)][string]$Workspace
    )
    Ensure-OrchestrateurDaemonToken | Out-Null
    try {
        $health = Invoke-RestMethod -Uri "http://127.0.0.1:28790/health" -TimeoutSec 2 -ErrorAction Stop
        Write-Host "Daemon déjà actif (version $($health.version))"
        return $true
    } catch {
        # pas encore démarré
    }
    $workDir = Split-Path -Parent $CliExe
    Start-Process -FilePath $CliExe -ArgumentList @("daemon", "run", "--workspace", $Workspace) -WorkingDirectory $workDir -WindowStyle Hidden
    Start-Sleep -Seconds 2
    try {
        $health = Invoke-RestMethod -Uri "http://127.0.0.1:28790/health" -TimeoutSec 5 -ErrorAction Stop
        Write-Host "Daemon démarré : http://127.0.0.1:28790/health (v$($health.version))" -ForegroundColor Green
        return $true
    } catch {
        Write-Warning "Daemon lancé mais health check en attente - vérifiez : orch doctor --workspace `"$Workspace`""
        return $false
    }
}

function Write-OrchestrateurMcpSnippet {
    param([Parameter(Mandatory = $true)][string]$Workspace)
    Write-Host ""
    Write-Host "MCP (Cursor / Claude Code) :" -ForegroundColor Cyan
    Write-Host "  orch mcp serve --workspace `"$Workspace`""
    Write-Host "  Puis ajoutez le serveur stdio MCP pointant vers la commande ci-dessus."
}

function Complete-OrchestrateurPostInstall {
    param(
        [string]$PreferredRoot = "",
        [string]$Workspace = "",
        [switch]$InstallDaemon,
        [switch]$SkipDoctor,
        [switch]$StartDaemon
    )
    . (Join-Path $PSScriptRoot "cli.ps1")
    Refresh-OrchestrateurUserPath

    $cli = Resolve-OrchestrateurCliExe -PreferredRoot $PreferredRoot
    if (-not $cli) {
        Write-Warning "Post-install partiel - orch introuvable. Ouvrez un nouveau terminal puis : orch doctor"
        return
    }
    Write-Host "CLI : $cli"

    if ([string]::IsNullOrWhiteSpace($Workspace)) {
        $Workspace = Get-OrchestrateurUserWorkspace
    }
    $example = ""
    if ($PreferredRoot) {
        $example = Join-Path $PreferredRoot "workspace\config\orchestrator.toml.example"
    }
    if (-not (Test-Path $example)) {
        $installed = Split-Path -Parent $cli
        $example = Join-Path $installed "workspace\orchestrator.toml.example"
    }
    Initialize-OrchestrateurUserWorkspace -ExampleConfig $example | Out-Null
    Ensure-OrchestrateurDaemonToken | Out-Null

    if (-not $SkipDoctor) {
        Invoke-OrchestrateurDoctor -CliExe $cli -Workspace $Workspace | Out-Null
    }
    if ($InstallDaemon) {
        Install-OrchestrateurDaemonScheduledTask -CliExe $cli -Workspace $Workspace | Out-Null
    }
    if ($StartDaemon) {
        Start-OrchestrateurDaemonNow -CliExe $cli -Workspace $Workspace | Out-Null
    }

    Write-OrchestrateurMcpSnippet -Workspace $Workspace
    Write-OrchestrateurCliUsageHint -Workspace $Workspace
    Write-Host ""
    Write-Host "Harness pret. Workspace : $Workspace"
}