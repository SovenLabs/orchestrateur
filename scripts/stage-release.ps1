# Prepare dist/staging for zip or Inno Setup installer.
param(
    [string]$StagingDir = "",
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

if (-not $Version) {
    $Version = (Select-String -Path "Cargo.toml" -Pattern '^version = "(.+)"' | ForEach-Object { $_.Matches[0].Groups[1].Value })
}
if (-not $StagingDir) {
    $StagingDir = Join-Path $Root "dist\staging"
}

$CliExe = Join-Path $Root "target\release\orchestrateur.exe"
$TuiExe = Join-Path $Root "target\release\orchestrateur-tui.exe"
$HudExe = Join-Path $Root "target\release\orchestrateur-hud.exe"
if (-not (Test-Path $CliExe)) { throw "Missing binary: $CliExe (run cargo build --release first)" }
if (-not (Test-Path $TuiExe)) { throw "Missing binary: $TuiExe (run cargo build --release first)" }
if (-not (Test-Path $HudExe)) { throw "Missing binary: $HudExe (run cargo build --release first)" }

Write-Host "Staging v$Version -> $StagingDir"
if (Test-Path $StagingDir) { Remove-Item -Recurse -Force $StagingDir }
New-Item -ItemType Directory -Path $StagingDir | Out-Null
$WsDir = Join-Path $StagingDir "workspace"
New-Item -ItemType Directory -Path (Join-Path $WsDir "config") | Out-Null
New-Item -ItemType Directory -Path (Join-Path $WsDir "memories") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $WsDir "logs") -Force | Out-Null

Copy-Item $CliExe $StagingDir
Copy-Item $TuiExe $StagingDir
Copy-Item $HudExe $StagingDir
Copy-Item "workspace\config\orchestrator.toml.example" (Join-Path $WsDir "orchestrator.toml.example")
Copy-Item "README.md" $StagingDir

@"
Orchestrateur v$Version - Windows package

Installed binaries (Program Files):
  orchestrateur.exe      CLI headless
  orchestrateur-tui.exe  interface terminal (ratatui)
  orchestrateur-hud.exe  interface graphique (egui)

User workspace (memories, LanceDB, config):
  %APPDATA%\Orchestrateur\workspace

First run:
  1. Edit orchestrator.toml in %APPDATA%\Orchestrateur\workspace\config\
  2. Launch Orchestrateur from Start menu

Providers: Ollama (local), xAI via XAI_API_KEY

Docs: https://github.com/SovenLabs/orchestrateur
"@ | Set-Content -Path (Join-Path $StagingDir "INSTALL.txt") -Encoding UTF8

@"
MIT License

Copyright (c) 2026 Soven

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the Software), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED AS IS, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
"@ | Set-Content -Path (Join-Path $StagingDir "LICENSE.txt") -Encoding UTF8

$IconPath = Join-Path $StagingDir "app.ico"
& (Join-Path $PSScriptRoot "generate-app-icon.ps1") -OutputPath $IconPath

Write-Host "Staging OK: $StagingDir"