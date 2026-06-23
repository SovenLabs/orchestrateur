# Export standalone Boule de Pixels Vivante (Godot 4.7)
param(
    [string]$GodotExe = "",
    [string]$OutputDir = "$PSScriptRoot\..\dist\sphere"
)

$ErrorActionPreference = "Stop"
$project = Join-Path $PSScriptRoot "..\godot-project\project.godot"
$scene = "res://scenes/SphereDedicated.tscn"

if (-not (Test-Path $project)) {
    Write-Error "Projet Godot introuvable: $project"
}

if ([string]::IsNullOrWhiteSpace($GodotExe)) {
    $candidates = @(
        "$env:LOCALAPPDATA\Godot\Godot_v4.7-stable_win64.exe",
        "C:\Program Files\Godot\Godot_v4.7-stable_win64.exe",
        "godot"
    )
    foreach ($c in $candidates) {
        if ($c -eq "godot") {
            $cmd = Get-Command godot -ErrorAction SilentlyContinue
            if ($cmd) { $GodotExe = $cmd.Source; break }
        } elseif (Test-Path $c) {
            $GodotExe = $c
            break
        }
    }
}

if ([string]::IsNullOrWhiteSpace($GodotExe)) {
    Write-Host "Godot 4.7 introuvable. Installez Godot ou passez -GodotExe."
    Write-Host "Lancement dev: ouvrir godot-project et F6 sur scenes/SphereDedicated.tscn"
    exit 1
}

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$preset = Join-Path $OutputDir "export_presets.cfg"
@"
[preset.0]
name="Windows Sphere"
platform="Windows Desktop"
runnable=true
export_path="./OrchestrateurSphere.exe"
custom_features=""
export_filter="resources"
include_filter="$scene"
exclude_filter=""
export_path_features=PackedStringArray()
"@ | Set-Content -Path $preset -Encoding UTF8

Write-Host "Export sphere -> $OutputDir"
& $GodotExe --headless --path (Split-Path $project) --export-release "Windows Sphere" (Join-Path $OutputDir "OrchestrateurSphere.exe")
Write-Host "Terminé."