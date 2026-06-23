# Génère les textures précalculées ebruneton pour le rendu TraceRay.
# Prérequis : submodule vendor/black_hole_shader, GNU make, g++.
param(
    [string]$VendorRoot = "$PSScriptRoot\..\apps\tauri-desktop\vendor\black_hole_shader",
    [string]$OutDir = "$PSScriptRoot\..\apps\tauri-desktop\public\cosmic\precomputed"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $VendorRoot)) {
    Write-Host "Submodule manquant : $VendorRoot"
    Write-Host "git submodule add https://github.com/ebruneton/black_hole_shader.git $VendorRoot"
    exit 1
}

Push-Location $VendorRoot
try {
    make
} finally {
    Pop-Location
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Write-Host "Copier les .dat générés vers $OutDir (ajuster selon sortie make)"
Write-Host "Assets optionnels — le shader light++ fonctionne sans ebruneton."