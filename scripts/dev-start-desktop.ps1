# Démarre l'interface Tauri (daemon requis dans un autre terminal)
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Desktop = Join-Path $Root "apps\tauri-desktop"
Set-Location $Desktop

$env:Path = "C:\Program Files\nodejs;$env:USERPROFILE\.cargo\bin;$env:LOCALAPPDATA\Microsoft\WinGet\Links;" + $env:Path

if (-not (Test-Path "node_modules")) {
    Write-Host "npm install..."
    & npm.cmd install
}

Write-Host "Lancement Tauri dev (WebView2 requis)..."
& npm.cmd run tauri dev