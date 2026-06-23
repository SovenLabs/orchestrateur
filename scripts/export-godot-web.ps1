# Export Godot TerritoryEmbed → apps/tauri-desktop/public/godot/
param(
    [string]$GodotExe = $env:GODOT4,
    [string]$PresetName = "Web Embed"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Project = Join-Path $Root "territoire-graphique\godot-project"
$OutDir = Join-Path $Root "apps\tauri-desktop\public\godot"

if (-not $GodotExe -or -not (Test-Path $GodotExe)) {
    $candidates = @(
        "C:\Program Files\Godot\Godot_v4.4-stable_win64.exe",
        "C:\Program Files\Godot\Godot_v4.7-stable_win64.exe",
        "godot4",
        "godot"
    )
    foreach ($c in $candidates) {
        if (Get-Command $c -ErrorAction SilentlyContinue) {
            $GodotExe = (Get-Command $c).Source
            break
        }
        if (Test-Path $c) {
            $GodotExe = $c
            break
        }
    }
}

if (-not $GodotExe -or -not (Test-Path $GodotExe)) {
    Write-Error "Godot 4 introuvable. Définissez `$env:GODOT4 ou installez Godot 4.x."
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

Write-Host "Export $PresetName depuis $Project"
& $GodotExe --headless --path $Project --export-release $PresetName $OutDir\index.html
if ($LASTEXITCODE -ne 0) {
    Write-Error "Export Godot échoué (code $LASTEXITCODE)."
}

Write-Host "OK → $OutDir"