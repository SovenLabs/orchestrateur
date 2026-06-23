# Génère icon.ico pour Tauri desktop à partir du script racine.
$ErrorActionPreference = "Stop"
$root = Resolve-Path (Join-Path $PSScriptRoot "..\..\..")
$outDir = Join-Path $PSScriptRoot "..\src-tauri\icons"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$iconIco = Join-Path $outDir "icon.ico"
& (Join-Path $root "scripts\generate-app-icon.ps1") -OutputPath $iconIco
Copy-Item $iconIco (Join-Path $outDir "32x32.png") -ErrorAction SilentlyContinue
Write-Host "Icône Tauri prête: $iconIco"
Write-Host "Pour le set complet: cd apps/tauri-desktop && npx tauri icon path/to/1024.png"