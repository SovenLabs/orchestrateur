# Package Windows (zip) — alias vers build-installer -ZipOnly
# Usage: .\scripts\package-windows.ps1

$ErrorActionPreference = "Stop"
& (Join-Path $PSScriptRoot "build-installer.ps1") -ZipOnly @args
exit $LASTEXITCODE