# Scan environnement Orchestrateur v2 — prérequis machine + état runtime
$ErrorActionPreference = "Continue"
$Root = Split-Path -Parent $PSScriptRoot

$env:Path = "$env:USERPROFILE\.cargo\bin;C:\Program Files\nodejs;$env:LOCALAPPDATA\Microsoft\WinGet\Links;" + $env:Path

function Test-Tool {
    param([string]$Name, [scriptblock]$Check, [string]$Required = "requis")
    $ok = $false
    $detail = ""
    try { $result = & $Check; $ok = $true; $detail = $result } catch { $detail = $_.Exception.Message }
    [PSCustomObject]@{
        Outil    = $Name
        Statut   = if ($ok) { "OK" } else { "MANQUANT" }
        Priorite = $Required
        Detail   = $detail
    }
}

Write-Host "`n=== Orchestrateur v2 - scan environnement ===" -ForegroundColor Cyan
Write-Host "Projet : $Root`n"

$checks = @(
    (Test-Tool "Rust (cargo)" { (& cargo --version 2>&1) -join " " })
    (Test-Tool "Node.js" { node --version })
    (Test-Tool "npm" { npm.cmd --version })
    (Test-Tool "MSVC link.exe" {
        $link = Get-ChildItem "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\*\VC\Tools\MSVC\*\bin\Hostx64\x64\link.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
        if (-not $link) { throw "link.exe introuvable - installez Visual Studio Build Tools 2022 (C++ workload)" }
        $link.FullName
    })
    (Test-Tool "WebView2 Runtime" {
        $pv = (Get-ItemProperty 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' -ErrorAction SilentlyContinue).pv
        if (-not $pv) { throw "WebView2 non détecté" }
        "v$pv"
    })
    (Test-Tool "just (optionnel)" { just --version } -Required "optionnel")
    (Test-Tool "Godot 4.x (optionnel)" {
        $godot = Get-Command godot, godot4 -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($godot) { return $godot.Source }
        $paths = @(
            "$env:LOCALAPPDATA\Programs\Godot\Godot*.exe",
            "$env:ProgramFiles\Godot\Godot*.exe",
            "$env:LOCALAPPDATA\Microsoft\WinGet\Packages\GodotEngine.GodotEngine*\Godot*.exe"
        )
        foreach ($p in $paths) {
            $found = Get-Item $p -ErrorAction SilentlyContinue | Select-Object -First 1
            if ($found) { return $found.FullName }
        }
        throw "Godot introuvable - requis pour Sphere / Territoire Godot"
    } -Required "optionnel")
)

$checks | Format-Table -AutoSize

. (Join-Path $PSScriptRoot "lib\cli.ps1")

Write-Host "`n--- Binaires projet ---" -ForegroundColor Yellow
@(
    (Get-OrchestrateurCliExe -Root $Root -Profile release),
    (Join-Path $Root "target\debug\orchestrateur-desktop.exe"),
    (Join-Path $Root "workspace\config\orchestrator.toml")
) | ForEach-Object {
    $exists = Test-Path $_
    Write-Host ("  [{0}] {1}" -f $(if ($exists) { "OK" } else { "--" }), $_)
}

Write-Host "`n--- Runtime ---" -ForegroundColor Yellow
try {
    $health = Invoke-RestMethod -Uri "http://127.0.0.1:28790/health" -TimeoutSec 2
    Write-Host "  Daemon : OK ($($health.version)) - clients=$($health.connected_clients)"
} catch {
    Write-Host "  Daemon : ARRÊTÉ (lancez scripts\dev-start-daemon.ps1)"
}

$desktop = Get-Process -Name "orchestrateur-desktop" -ErrorAction SilentlyContinue
if ($desktop) {
    Write-Host "  Tauri desktop : $($desktop.Count) processus"
} else {
    Write-Host "  Tauri desktop : ARRÊTÉ (lancez scripts\dev-start-desktop.ps1)"
}

$missing = $checks | Where-Object { $_.Statut -eq "MANQUANT" -and $_.Priorite -eq "requis" }
if ($missing) {
    Write-Host "`nACTION : installer les outils requis manquants." -ForegroundColor Red
    exit 1
}

$optional = $checks | Where-Object { $_.Statut -eq "MANQUANT" -and $_.Priorite -eq "optionnel" }
if ($optional) {
    Write-Host "`nOptionnel manquant : $($optional.Outil -join ', ')" -ForegroundColor DarkYellow
    Write-Host "  winget install Casey.Just"
    Write-Host "  winget install GodotEngine.GodotEngine"
}

Write-Host "`nLancement recommandé :" -ForegroundColor Green
Write-Host "  Install CLI : .\install.ps1 -Dev"
Write-Host "  Terminal 1 : .\scripts\dev-start-daemon.ps1  (ou : orch daemon run --workspace workspace)"
Write-Host "  Terminal 2 : .\scripts\dev-start-desktop.ps1"
Write-Host "  DevTools   : `$env:ORCHESTRATEUR_DEVTOOLS='1' avant desktop (optionnel)`n"