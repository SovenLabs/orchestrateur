# Crée un tag Git annoté pour une phase Orchestrateur v2
param(
    [Parameter(Mandatory = $true)]
    [int]$Phase,
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

if (-not $Version) {
    $Version = (Select-String -Path "Cargo.toml" -Pattern '^version = "(.+)"' | ForEach-Object { $_.Matches[0].Groups[1].Value })
}

$Tag = "phase$Phase-v$Version"
$Message = "Phase $Phase — Orchestrateur v$Version (hybride Tauri + Rust + Godot)"

$existing = git tag -l $Tag
if ($existing) {
    Write-Host "Tag existant : $Tag"
    git show $Tag --no-patch
    exit 0
}

git tag -a $Tag -m $Message
Write-Host "Tag créé : $Tag"
Write-Host "Push : git push origin $Tag"