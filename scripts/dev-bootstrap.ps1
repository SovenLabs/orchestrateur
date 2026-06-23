# Bootstrap dev — build + start daemon (Windows)
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$env:Path = "$env:USERPROFILE\.cargo\bin;C:\Program Files\nodejs;" + $env:Path

# MSVC linker (requis pour cargo build sur Windows)
$vcvars = @(
    "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
    "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
) | Where-Object { Test-Path $_ } | Select-Object -First 1
if ($vcvars) {
    cmd /c "`"$vcvars`" && set" | ForEach-Object {
        if ($_ -match '^(.*?)=(.*)$') { Set-Item -Path "env:$($matches[1])" -Value $matches[2] }
    }
}

$config = Join-Path $Root "workspace\config\orchestrator.toml"
if (-not (Test-Path $config)) {
    Copy-Item (Join-Path $Root "workspace\config\orchestrator.toml.example") $config
    Write-Host "Config créée : $config"
}

$exe = Join-Path $Root "target\release\orchestrateur.exe"
if (-not (Test-Path $exe)) {
    Write-Host "Compilation release (première fois, plusieurs minutes)..."
    cargo build --release -p orchestrateur-cli
}

$env:ORCHESTRATEUR_DAEMON_TOKEN = "dev"
Write-Host "Démarrage daemon sur http://127.0.0.1:28790/health"
& $exe daemon run --workspace workspace