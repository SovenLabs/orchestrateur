# Build release + package zip + installateur Inno Setup (Setup.exe)
# Usage:
#   .\scripts\build-installer.ps1
#   .\scripts\build-installer.ps1 -SkipBuild
#   .\scripts\build-installer.ps1 -ZipOnly

param(
    [switch]$SkipBuild,
    [switch]$ZipOnly,
    [switch]$InstallInno
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$Version = (Select-String -Path "Cargo.toml" -Pattern '^version = "(.+)"' | ForEach-Object { $_.Matches[0].Groups[1].Value })
$DistDir = Join-Path $Root "dist"
$StagingDir = Join-Path $DistDir "staging"
$PackageName = "Orchestrateur-v$Version-win64"

function Find-InnoCompiler {
    $candidates = @(
        "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
        "$env:ProgramFiles\Inno Setup 6\ISCC.exe",
        "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe"
    )
    foreach ($path in $candidates) {
        if (Test-Path -LiteralPath $path) { return $path }
    }
    $cmd = Get-Command ISCC.exe -ErrorAction SilentlyContinue
    if ($cmd -and (Test-Path -LiteralPath $cmd.Source)) { return $cmd.Source }
    return $null
}

function Ensure-InnoSetup {
    $iscc = Find-InnoCompiler
    if ($iscc) { return $iscc }

    if (-not $InstallInno) {
        Write-Host "Inno Setup 6 introuvable. Relancez avec -InstallInno ou installez JRSoftware.InnoSetup"
        return $null
    }

    Write-Host "Installation Inno Setup 6 via winget..."
    winget install --id JRSoftware.InnoSetup --accept-package-agreements --accept-source-agreements --silent | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "Echec installation Inno Setup (winget exit $LASTEXITCODE)"
    }
    Start-Sleep -Seconds 2
    $iscc = Find-InnoCompiler
    if (-not $iscc) {
        throw "Inno Setup installe mais ISCC.exe introuvable"
    }
    return $iscc
}

if (-not $SkipBuild) {
    Write-Host "Compilation release (CLI + daemon + gateway)..."
    cargo build --release -p orchestrateur-cli --features http,gateway,websocket-server
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

. (Join-Path $PSScriptRoot "stage-release.ps1") -StagingDir $StagingDir -Version $Version | Out-Null

$ZipPath = Join-Path $DistDir "$PackageName.zip"
if (Test-Path $ZipPath) { Remove-Item -Force $ZipPath }
Compress-Archive -Path $StagingDir -DestinationPath $ZipPath -Force
Write-Host "ZIP: $ZipPath"

if ($ZipOnly) {
    Write-Host "Termine (zip uniquement)."
    exit 0
}

$Iscc = Ensure-InnoSetup
if (-not $Iscc) {
    Write-Host "Setup.exe non produit - ZIP disponible: $ZipPath"
    exit 0
}

$IssPath = Join-Path $Root "installer\orchestrateur.iss"
Write-Host "Compilation installateur Inno Setup..."
& $Iscc "/DStagingRoot=$StagingDir" "/DMyAppVersion=$Version" "/DMyAppVersionFull=$Version.0" $IssPath

if ($LASTEXITCODE -ne 0) {
    throw "ISCC a echoue (exit $LASTEXITCODE)"
}

$SetupExe = Join-Path $DistDir "Orchestrateur-v$Version-Setup-win64.exe"
if (Test-Path $SetupExe) {
    Write-Host "SETUP: $SetupExe"
} else {
    throw "Setup.exe attendu introuvable: $SetupExe"
}