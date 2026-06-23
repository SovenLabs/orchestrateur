# Prepare dist/staging for zip or Inno Setup installer.
param(
    [string]$StagingDir = "",
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root
. (Join-Path $PSScriptRoot "lib\cli.ps1")

if (-not $Version) {
    $Version = (Select-String -Path "Cargo.toml" -Pattern '^version = "(.+)"' | ForEach-Object { $_.Matches[0].Groups[1].Value })
}
if (-not $StagingDir) {
    $StagingDir = Join-Path $Root "dist\staging"
}

if (-not (Test-OrchestrateurCliBuilt -Root $Root -Profile release)) {
    throw "Binaire CLI manquant (orch.exe). Lancez : cargo build --release -p orchestrateur-cli --bin orch"
}

Write-Host "Staging v$Version -> $StagingDir"
if (Test-Path $StagingDir) { Remove-Item -Recurse -Force $StagingDir }
New-Item -ItemType Directory -Path $StagingDir | Out-Null
$WsDir = Join-Path $StagingDir "workspace"
New-Item -ItemType Directory -Path (Join-Path $WsDir "config") | Out-Null
New-Item -ItemType Directory -Path (Join-Path $WsDir "memories") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $WsDir "logs") -Force | Out-Null

Copy-Item (Get-OrchestrateurCliExe -Root $Root -Profile release) $StagingDir
Write-OrchestrateurCliShims -BinDir $StagingDir
Copy-Item "workspace\config\orchestrator.toml.example" (Join-Path $WsDir "orchestrator.toml.example")
Copy-Item "README.md" $StagingDir
Copy-Item "territoire-graphique\communication.md" $StagingDir

@"
Orchestrateur v$Version - Windows package

Installed binaries (Program Files) :
  orchestrateur.exe, orchestre.exe, orch.exe (meme binaire)
  CLI headless + daemon Territoire Graphique

User workspace (memories, LanceDB, config):
  %APPDATA%\Orchestrateur\workspace

First run:
  1. Edit orchestrator.toml in %APPDATA%\Orchestrateur\workspace\config\
  2. Start daemon: orch daemon run --workspace <path>
  3. Launch Desktop Tauri or Territoire Graphique (Godot) — Phases 21–26
  4. Package hybride complet : just release-v26

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

Write-Host "Staging complete: $StagingDir"