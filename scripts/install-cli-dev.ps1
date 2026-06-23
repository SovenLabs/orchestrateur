# Redirige vers l'installateur unique (install.ps1 -Dev).
# Conservé pour compatibilité : just install-cli, anciennes docs.
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
& (Join-Path $Root "install.ps1") -Dev @args
exit $LASTEXITCODE