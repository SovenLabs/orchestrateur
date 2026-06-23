# ============================================================================
# Orchestrateur Installer for Windows (Hermes-style)
# ============================================================================
#
# Usage:
#   iex (irm https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/scripts/install.ps1)
#
# Dev (compile from clone):
#   .\scripts\install.ps1 -Dev
#
# Stage protocol (GUI / CI drivers):
#   .\scripts\install.ps1 -Manifest
#   .\scripts\install.ps1 -Stage git
#
# ============================================================================

#Requires -Version 5.1

param(
    [switch]$Dev,
    [switch]$Debug,
    [switch]$SkipBuild,
    [switch]$InstallDaemon,
    [switch]$SkipDoctor,
    [switch]$StartDaemon,
    [switch]$AllUsers,
    [string]$Version = "",
    [string]$Branch = "main",
    [string]$Commit = "",
    [string]$Tag = "",
    [string]$OrchestrateurHome = $(if ($env:ORCHESTRATEUR_HOME) { $env:ORCHESTRATEUR_HOME } else { "$env:LOCALAPPDATA\orchestrateur" }),
    [string]$InstallDir = $(if ($env:ORCHESTRATEUR_HOME) { "$env:ORCHESTRATEUR_HOME\orchestrateur" } else { "$env:LOCALAPPDATA\orchestrateur\orchestrateur" }),

    [switch]$Manifest,
    [string]$Stage,
    [switch]$ProtocolVersion,
    [switch]$NonInteractive,
    [switch]$Json
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

try {
    [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new()
} catch { }

# ============================================================================
# Configuration
# ============================================================================

$Repo = "SovenLabs/orchestrateur"
$RepoUrlHttps = "https://github.com/$Repo.git"
$RawBase = "https://raw.githubusercontent.com/$Repo/main"
$InstallStageProtocolVersion = 1

if ($env:ORCHESTRATEUR_DEV -eq "1") { $Dev = $true }
if ($env:ORCHESTRATEUR_VERSION) { $Version = $env:ORCHESTRATEUR_VERSION }
if ($env:ORCHESTRATEUR_INSTALL_DAEMON -eq "1") { $InstallDaemon = $true }
if ($env:ORCHESTRATEUR_SKIP_DOCTOR -eq "1") { $SkipDoctor = $true }
if ($env:ORCHESTRATEUR_START_DAEMON -eq "1") { $StartDaemon = $true }
if ($env:ORCHESTRATEUR_ALL_USERS -eq "1") { $AllUsers = $true }

$script:UsedReleaseBinary = $false
$script:UsedSourceBuild = $false
$script:_StageSkippedReason = $null

# ============================================================================
# Helpers
# ============================================================================

function Write-Banner {
    Write-Host ""
    Write-Host "+---------------------------------------------------------+" -ForegroundColor Magenta
    Write-Host "|           * Orchestrateur Installer                     |" -ForegroundColor Magenta
    Write-Host "+---------------------------------------------------------+" -ForegroundColor Magenta
    Write-Host "|  Second cerveau local souverain - Soven Labs            |" -ForegroundColor Magenta
    Write-Host "+---------------------------------------------------------+" -ForegroundColor Magenta
    Write-Host ""
}

function Write-Info {
    param([string]$Message)
    Write-Host "-> $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "[OK] $Message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[!] $Message" -ForegroundColor Yellow
}

function Write-Err {
    param([string]$Message)
    Write-Host "[X] $Message" -ForegroundColor Red
}

function Sync-EnvPath {
    $env:Path = [Environment]::GetEnvironmentVariable("Path", "User") + ";" +
        [Environment]::GetEnvironmentVariable("Path", "Machine")
}

function Get-WindowsArch {
    try {
        $proc = Get-CimInstance -ClassName Win32_Processor -ErrorAction Stop |
            Select-Object -First 1
        switch ([int]$proc.Architecture) {
            12 { return "arm64" }
            9  { return "x64" }
            0  { return "x86" }
            5  { return "arm" }
        }
    } catch { }

    $envArch = if ($env:PROCESSOR_ARCHITEW6432) { $env:PROCESSOR_ARCHITEW6432 } else { $env:PROCESSOR_ARCHITECTURE }
    switch ($envArch) {
        "ARM64" { return "arm64" }
        "AMD64" { return "x64" }
        "x86"   { return "x86" }
        default {
            if ([Environment]::Is64BitOperatingSystem) { return "x64" } else { return "x86" }
        }
    }
}

function Get-PowerShellHostExe {
    try {
        $hostExe = (Get-Process -Id $PID).Path
        if ($hostExe -and (Test-Path $hostExe)) {
            $leaf = Split-Path $hostExe -Leaf
            if ($leaf -match '^(?i:powershell|pwsh)\.exe$') { return $hostExe }
        }
    } catch { }
    foreach ($candidate in @("powershell", "pwsh")) {
        $cmd = Get-Command $candidate -CommandType Application -ErrorAction SilentlyContinue |
            Select-Object -First 1
        if ($cmd -and $cmd.Source) { return $cmd.Source }
    }
    return "powershell"
}

function Import-OrchestrateurInstallLibs {
    $candidates = @()
    if ($PSScriptRoot) {
        $candidates += Join-Path $PSScriptRoot "lib"
        $candidates += Join-Path (Split-Path -Parent $PSScriptRoot) "scripts\lib"
    }
    foreach ($dir in $candidates) {
        $post = Join-Path $dir "post-install.ps1"
        $cli = Join-Path $dir "cli.ps1"
        if ((Test-Path -LiteralPath $post) -and (Test-Path -LiteralPath $cli)) {
            . $cli
            . $post
            return $dir
        }
    }

    $libDir = Join-Path $env:TEMP "orchestrateur-install-lib"
    if (-not (Test-Path $libDir)) {
        New-Item -ItemType Directory -Path $libDir -Force | Out-Null
    }
    foreach ($name in @("cli.ps1", "post-install.ps1")) {
        $dest = Join-Path $libDir $name
        Invoke-WebRequest -Uri "$RawBase/scripts/lib/$name" -OutFile $dest -UseBasicParsing
        . $dest
    }
    return $libDir
}

function Ensure-OrchestrateurHomeDirs {
    foreach ($dir in @($OrchestrateurHome, (Join-Path $OrchestrateurHome "bin"), (Join-Path $OrchestrateurHome "git"))) {
        if (-not (Test-Path $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
        }
    }
}

function Add-GitToSessionPath {
    $gitDirs = @(
        (Join-Path $OrchestrateurHome "git\cmd"),
        (Join-Path $OrchestrateurHome "git\bin"),
        (Join-Path $OrchestrateurHome "git\usr\bin")
    )
    foreach ($dir in $gitDirs) {
        if (Test-Path $dir) {
            if ($env:Path -notlike "*$dir*") {
                $env:Path = "$dir;$env:Path"
            }
        }
    }
}

function Set-GitBashEnvVar {
    $candidates = @(
        (Join-Path $OrchestrateurHome "git\bin\bash.exe"),
        (Join-Path $OrchestrateurHome "git\usr\bin\bash.exe")
    )
    foreach ($bash in $candidates) {
        if (Test-Path $bash) {
            [Environment]::SetEnvironmentVariable("ORCHESTRATEUR_GIT_BASH_PATH", $bash, "User")
            return
        }
    }
}

function Install-Git {
    Write-Info "Checking Git..."
    Add-GitToSessionPath

    if (Get-Command git -ErrorAction SilentlyContinue) {
        $version = git --version
        Write-Success "Git found ($version)"
        Set-GitBashEnvVar
        return $true
    }

    Write-Info "Git not found - downloading PortableGit to $OrchestrateurHome\git ..."
    Ensure-OrchestrateurHomeDirs

    try {
        $arch = Get-WindowsArch
        $gitTag = "v2.54.0.windows.1"
        $gitVer = "2.54.0"

        if ($arch -eq "arm64") {
            $assetName = "PortableGit-$gitVer-arm64.7z.exe"
            $downloadIsZip = $false
        } elseif ($arch -eq "x64") {
            $assetName = "PortableGit-$gitVer-64-bit.7z.exe"
            $downloadIsZip = $false
        } else {
            Write-Warn "32-bit Windows - installing MinGit (bash features unavailable)"
            $assetName = "MinGit-$gitVer-32-bit.zip"
            $downloadIsZip = $true
        }

        $downloadUrl = "https://github.com/git-for-windows/git/releases/download/$gitTag/$assetName"
        $tmpFile = Join-Path $env:TEMP $assetName
        $gitDir = Join-Path $OrchestrateurHome "git"

        Invoke-WebRequest -Uri $downloadUrl -OutFile $tmpFile -UseBasicParsing

        if (Test-Path $gitDir) {
            Remove-Item -Recurse -Force $gitDir
        }
        New-Item -ItemType Directory -Path $gitDir -Force | Out-Null

        if ($downloadIsZip) {
            Expand-Archive -Path $tmpFile -DestinationPath $gitDir -Force
        } else {
            $extractProc = Start-Process -FilePath $tmpFile `
                -ArgumentList "-o`"$gitDir`"", "-y" `
                -NoNewWindow -Wait -PassThru
            if ($extractProc.ExitCode -ne 0) {
                throw "PortableGit extraction failed (exit $($extractProc.ExitCode))"
            }
        }
        Remove-Item -Force $tmpFile -ErrorAction SilentlyContinue

        $gitExe = Join-Path $gitDir "cmd\git.exe"
        if (-not (Test-Path $gitExe)) {
            throw "Git extraction did not produce git.exe at $gitExe"
        }

        $gitCmd = Join-Path $gitDir "cmd"
        $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($currentPath -notlike "*$gitCmd*") {
            [Environment]::SetEnvironmentVariable("Path", "$gitCmd;$currentPath", "User")
        }
        Add-GitToSessionPath
        Set-GitBashEnvVar

        $version = & $gitExe --version
        Write-Success "PortableGit installed ($version)"
        return $true
    } catch {
        Write-Err "Failed to install Git: $_"
        Write-Info "Install manually: https://git-scm.com/download/win"
        return $false
    }
}

function Test-Rust {
    Sync-EnvPath
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    if (Test-Path $cargoBin) {
        if ($env:Path -notlike "*$cargoBin*") {
            $env:Path = "$cargoBin;$env:Path"
        }
    }
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        $ver = cargo --version
        Write-Success "Rust found ($ver)"
        return $true
    }
    return $false
}

function Install-Rust {
    if (Test-Rust) { return $true }

    Write-Info "Installing rustup (no admin required)..."
    $arch = Get-WindowsArch
    $rustupUrl = switch ($arch) {
        "arm64" { "https://win.rustup.rs/aarch64" }
        "x86"   { "https://win.rustup.rs/i686" }
        default { "https://win.rustup.rs/x86_64" }
    }

    $rustupExe = Join-Path $env:TEMP "rustup-init.exe"
    Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupExe -UseBasicParsing

    $prevEAP = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        & $rustupExe -y --default-toolchain stable 2>&1 | Out-Null
    } finally {
        $ErrorActionPreference = $prevEAP
    }
    Remove-Item -Force $rustupExe -ErrorAction SilentlyContinue

    if (Test-Rust) {
        Write-Success "Rust installed via rustup"
        return $true
    }

    Write-Err "rustup installation failed"
    Write-Info "Install manually: https://rustup.rs/"
    return $false
}

function Get-ResolvedGitRef {
    if ($Commit) { return $Commit }
    if ($Tag) { return $Tag }
    return $Branch
}

function Step-OutOfInstallDir {
    try {
        $currentResolved = (Get-Location).ProviderPath
        $installResolved = $null
        if (Test-Path $InstallDir) {
            $installResolved = (Resolve-Path $InstallDir -ErrorAction SilentlyContinue).ProviderPath
        }
        if ($installResolved -and $currentResolved.ToLower().StartsWith($installResolved.ToLower())) {
            Write-Info "Stepping out of $InstallDir ..."
            Set-Location $env:USERPROFILE
        }
    } catch { }
}

function Install-Repository {
    Write-Info "Installing repository to $InstallDir ..."
    Ensure-OrchestrateurHomeDirs
    Add-GitToSessionPath

    $gitCmd = Get-Command git -ErrorAction SilentlyContinue
    if (-not $gitCmd) {
        throw "git not available"
    }

    $ref = Get-ResolvedGitRef
    $didUpdate = $false

    if (Test-Path $InstallDir) {
        $repoValid = $false
        if (Test-Path (Join-Path $InstallDir ".git")) {
            Push-Location $InstallDir
            try {
                $global:LASTEXITCODE = 0
                $revParseOut = & git -c windows.appendAtomically=false rev-parse --is-inside-work-tree 2>&1
                $revParseOk = ($LASTEXITCODE -eq 0) -and ($revParseOut -match "true")

                $global:LASTEXITCODE = 0
                $null = & git -c windows.appendAtomically=false status --short 2>&1
                $statusOk = ($LASTEXITCODE -eq 0)

                $global:LASTEXITCODE = 0
                $null = & git -c windows.appendAtomically=false rev-parse --verify HEAD 2>&1
                $hasCommit = ($LASTEXITCODE -eq 0)

                if ($revParseOk -and $statusOk -and $hasCommit) {
                    $repoValid = $true
                }
            } catch { }
            Pop-Location
        }

        if ($repoValid) {
            Write-Info "Existing installation found, updating..."
            Push-Location $InstallDir
            $prevEAP = $ErrorActionPreference
            $ErrorActionPreference = "Continue"
            try {
                git -c windows.appendAtomically=false config core.autocrlf false 2>$null
                git -c windows.appendAtomically=false stash push -u -m "orchestrateur-install-autostash" 2>$null
                git -c windows.appendAtomically=false fetch origin $Branch 2>&1 | Out-Null
                if ($LASTEXITCODE -ne 0) { throw "git fetch failed (exit $LASTEXITCODE)" }
                if ($Commit -or $Tag) {
                    git -c windows.appendAtomically=false checkout $ref 2>&1 | Out-Null
                } else {
                    git -c windows.appendAtomically=false reset --hard "origin/$Branch" 2>&1 | Out-Null
                }
                if ($LASTEXITCODE -ne 0) { throw "git reset/checkout failed (exit $LASTEXITCODE)" }
                git -c windows.appendAtomically=false stash pop 2>$null
                $didUpdate = $true
            } finally {
                $ErrorActionPreference = $prevEAP
                Pop-Location
            }
            if ($didUpdate) {
                Write-Success "Repository updated ($ref)"
            }
            return $true
        }

        Write-Warn "Broken checkout at $InstallDir - re-cloning..."
        Step-OutOfInstallDir
        Remove-Item -Recurse -Force $InstallDir -ErrorAction SilentlyContinue
    }

    New-Item -ItemType Directory -Path (Split-Path $InstallDir -Parent) -Force | Out-Null
    $prevEAP = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        if ($Commit -or $Tag) {
            & git clone --branch $Branch --single-branch $RepoUrlHttps $InstallDir 2>&1 | Out-Null
            if ($LASTEXITCODE -ne 0) { throw "git clone failed (exit $LASTEXITCODE)" }
            Push-Location $InstallDir
            git -c windows.appendAtomically=false checkout $ref 2>&1 | Out-Null
            if ($LASTEXITCODE -ne 0) { throw "git checkout $ref failed (exit $LASTEXITCODE)" }
            Pop-Location
        } else {
            & git clone --branch $Branch $RepoUrlHttps $InstallDir 2>&1 | Out-Null
            if ($LASTEXITCODE -ne 0) { throw "git clone failed (exit $LASTEXITCODE)" }
        }
    } finally {
        $ErrorActionPreference = $prevEAP
    }

    Push-Location $InstallDir
    git -c windows.appendAtomically=false config core.autocrlf false 2>$null
    Pop-Location

    Write-Success "Repository cloned ($ref)"
    return $true
}

function Normalize-Version([string]$Value) {
    if ([string]::IsNullOrWhiteSpace($Value)) { return "" }
    $v = $Value.Trim()
    if ($v.StartsWith("v")) { $v = $v.Substring(1) }
    return $v
}

function Get-InstallVersion {
    if (-not [string]::IsNullOrWhiteSpace($Version)) {
        return (Normalize-Version $Version)
    }
    if (Test-Path (Join-Path $InstallDir "Cargo.toml")) {
        $match = Select-String -Path (Join-Path $InstallDir "Cargo.toml") -Pattern '^version = "(.+)"' |
            Select-Object -First 1
        if ($match) {
            return $match.Matches[0].Groups[1].Value
        }
    }
    return ""
}

function Get-LatestReleaseVersion {
    $headers = @{ "User-Agent" = "Orchestrateur-Install-Script" }
    $api = "https://api.github.com/repos/$Repo/releases/latest"
    try {
        $release = Invoke-RestMethod -Uri $api -Headers $headers -UseBasicParsing
        $tag = [string]$release.tag_name
        if ($tag.StartsWith("v")) { return $tag.Substring(1) }
        return $tag
    } catch {
        return $null
    }
}

function Test-ReleaseAssetExists {
    param([string]$Ver)
    $zipName = "Orchestrateur-v$Ver-win64.zip"
    $url = "https://github.com/$Repo/releases/download/v$Ver/$zipName"
    try {
        $resp = Invoke-WebRequest -Uri $url -Method Head -UseBasicParsing
        return ($resp.StatusCode -eq 200)
    } catch {
        return $false
    }
}

function Install-ReleaseBinary {
    $ver = Get-InstallVersion
    if ([string]::IsNullOrWhiteSpace($ver)) {
        Write-Info "Resolving latest GitHub release version..."
        $ver = Get-LatestReleaseVersion
    }
    if ([string]::IsNullOrWhiteSpace($ver)) {
        $script:_StageSkippedReason = "No GitHub release published yet"
        return
    }

    if (-not (Test-ReleaseAssetExists -Ver $ver)) {
        $script:_StageSkippedReason = "Release zip Orchestrateur-v$ver-win64.zip not found"
        return
    }

    $zipName = "Orchestrateur-v$ver-win64.zip"
    $downloadUrl = "https://github.com/$Repo/releases/download/v$ver/$zipName"
    $tempDir = Join-Path $env:TEMP "orchestrateur-release-$ver"
    $zipPath = Join-Path $tempDir $zipName

    if (Test-Path $tempDir) { Remove-Item -Recurse -Force $tempDir }
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

    Write-Info "Downloading $zipName ..."
    Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath -UseBasicParsing

    $extractDir = Join-Path $tempDir "extracted"
    Expand-Archive -Path $zipPath -DestinationPath $extractDir -Force

    $orchSrc = Get-ChildItem -Path $extractDir -Recurse -Filter "orch.exe" -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if (-not $orchSrc) {
        throw "orch.exe not found in $zipName"
    }

    $binDir = Get-OrchestrateurUserBinDir
    if (-not (Test-Path $binDir)) {
        New-Item -ItemType Directory -Path $binDir -Force | Out-Null
    }

    $orchDst = Join-Path $binDir "orch.exe"
    Copy-Item -LiteralPath $orchSrc.FullName -Destination $orchDst -Force
    Write-OrchestrateurCliShims -BinDir $binDir

    $script:UsedReleaseBinary = $true
    Write-Success "Release v$ver installed to $binDir"
}

function Install-Build {
    if (-not (Test-Path (Join-Path $InstallDir "Cargo.toml"))) {
        throw "Repository missing at $InstallDir - run repository stage first"
    }
    if (-not (Install-Rust)) {
        throw "Rust not available and auto-install failed"
    }

    Initialize-OrchestrateurBuildEnv
    $profile = if ($Debug) { "debug" } else { "release" }

    if (-not $SkipBuild) {
        Build-OrchestrateurCli -Root $InstallDir -Profile $profile
    } else {
        Remove-OrchestrateurLegacyCliArtifacts -Root $InstallDir -Profile $profile
    }

    if ($AllUsers) {
        Install-OrchestrateurCliToMachinePath -Root $InstallDir -Profile $profile
    } else {
        Install-OrchestrateurCliToUserPath -Root $InstallDir -Profile $profile
    }

    $script:UsedSourceBuild = $true
    Write-Success "Built from source ($profile)"
}

function Set-PathVariable {
    $binDir = Get-OrchestrateurUserBinDir
    Add-OrchestrateurPathEntry -Dir $binDir -Scope "User"

    $currentHome = [Environment]::GetEnvironmentVariable("ORCHESTRATEUR_HOME", "User")
    if (-not $currentHome -or $currentHome -ne $OrchestrateurHome) {
        [Environment]::SetEnvironmentVariable("ORCHESTRATEUR_HOME", $OrchestrateurHome, "User")
        Write-Success "Set ORCHESTRATEUR_HOME=$OrchestrateurHome"
    }
    $env:ORCHESTRATEUR_HOME = $OrchestrateurHome

    $gitCmd = Join-Path $OrchestrateurHome "git\cmd"
    if (Test-Path $gitCmd) {
        Add-OrchestrateurPathEntry -Dir $gitCmd -Scope "User"
    }

    Sync-EnvPath
    Write-Success "orch command ready in PATH"
}

function Initialize-WorkspaceTemplates {
    $workspace = Get-OrchestrateurUserWorkspace
    $example = Join-Path $InstallDir "workspace\config\orchestrator.toml.example"
    Initialize-OrchestrateurUserWorkspace -ExampleConfig $example | Out-Null
    Write-Success "Workspace ready: $workspace"
}

function Invoke-PostInstallStage {
    Complete-OrchestrateurPostInstall `
        -PreferredRoot $InstallDir `
        -InstallDaemon:$InstallDaemon `
        -SkipDoctor:$SkipDoctor `
        -StartDaemon:$StartDaemon
}

function Write-BootstrapMarker {
    if (-not (Test-Path $InstallDir)) {
        Write-Warn "Skipping bootstrap marker: $InstallDir missing"
        return
    }

    $pinnedCommit = $Commit
    if (-not $pinnedCommit) {
        $gitCmd = Get-Command git -ErrorAction SilentlyContinue
        if ($gitCmd) {
            Push-Location $InstallDir
            try {
                $resolved = & $gitCmd.Source rev-parse HEAD 2>$null
                if ($LASTEXITCODE -eq 0 -and $resolved) {
                    $pinnedCommit = $resolved.Trim()
                }
            } catch { }
            Pop-Location
        }
    }

    $marker = @{
        schema_version = 1
        installed_at   = (Get-Date).ToUniversalTime().ToString("o")
        pinned_commit  = $pinnedCommit
        pinned_branch  = $Branch
        install_mode   = if ($script:UsedReleaseBinary) { "release" } elseif ($script:UsedSourceBuild) { "build" } else { "unknown" }
        version        = Get-InstallVersion
    }
    $json = $marker | ConvertTo-Json -Compress
    $markerPath = Join-Path $OrchestrateurHome ".orchestrateur-bootstrap-complete"
    $utf8NoBom = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllText($markerPath, $json, $utf8NoBom)
    Write-Success "Bootstrap marker written: $markerPath"
}

function Invoke-OnboardWizard {
    if ($NonInteractive) {
        Write-Info "Skipping onboard wizard (non-interactive)"
        return
    }
    Sync-EnvPath
    $cli = Resolve-OrchestrateurCliExe -PreferredRoot $InstallDir
    if (-not $cli) {
        Write-Warn "orch not found - run onboard manually after opening a new terminal"
        return
    }
    $workspace = Get-OrchestrateurUserWorkspace
    Write-Info "Running orch onboard ..."
    & $cli onboard --workspace $workspace
}

function Write-Completion {
    $workspace = Get-OrchestrateurUserWorkspace
    Write-Host ""
    Write-Host "+---------------------------------------------------------+" -ForegroundColor Green
    Write-Host "|              [OK] Installation Complete!                |" -ForegroundColor Green
    Write-Host "+---------------------------------------------------------+" -ForegroundColor Green
    Write-Host ""
    Write-Host "* Your files:" -ForegroundColor Cyan
    Write-Host "   Workspace:  $workspace"
    Write-Host "   Code:       $InstallDir"
    Write-Host "   Home:       $OrchestrateurHome"
    Write-Host ""
    Write-Host "* Commands:" -ForegroundColor Cyan
    Write-Host "   orch doctor"
    Write-Host "   orch onboard"
    Write-Host "   orch daemon run --workspace `"$workspace`""
    Write-Host ""
    Write-Host "[*] Restart your terminal for PATH changes to take effect" -ForegroundColor Yellow
    Write-Host ""
}

# ============================================================================
# Stage protocol
# ============================================================================

$InstallStages = @(
    @{ Name = "git";              Title = "Installing Git";                    Category = "prereqs";      NeedsUserInput = $false; Worker = "Stage-Git" }
    @{ Name = "repository";       Title = "Cloning Orchestrateur repository";  Category = "install";      NeedsUserInput = $false; Worker = "Stage-Repository" }
    @{ Name = "release";           Title = "Installing release binary";       Category = "install";      NeedsUserInput = $false; Worker = "Stage-Release" }
    @{ Name = "build";             Title = "Building from source";            Category = "install";      NeedsUserInput = $false; Worker = "Stage-Build" }
    @{ Name = "path";              Title = "Adding orch to PATH";             Category = "finalize";     NeedsUserInput = $false; Worker = "Stage-Path" }
    @{ Name = "workspace";         Title = "Initializing user workspace";     Category = "finalize";     NeedsUserInput = $false; Worker = "Stage-Workspace" }
    @{ Name = "post-install";      Title = "Running doctor and daemon setup"; Category = "finalize";     NeedsUserInput = $false; Worker = "Stage-PostInstall" }
    @{ Name = "bootstrap-marker";  Title = "Marking install complete";        Category = "finalize";     NeedsUserInput = $false; Worker = "Stage-BootstrapMarker" }
    @{ Name = "onboard";           Title = "First-run onboard wizard";        Category = "post-install"; NeedsUserInput = $true;  Worker = "Stage-Onboard" }
)

function Stage-Git              { if (-not (Install-Git)) { throw "Git not available" } }
function Stage-Repository     { Install-Repository | Out-Null }
function Stage-Release        {
    if ($Dev) {
        $script:_StageSkippedReason = "Dev mode (-Dev) - using source build instead"
        return
    }
    Install-ReleaseBinary
}
function Stage-Build          {
    if ($script:UsedReleaseBinary) {
        $script:_StageSkippedReason = "Release binary already installed"
        return
    }
    Install-Build
}
function Stage-Path           { Set-PathVariable }
function Stage-Workspace      { Initialize-WorkspaceTemplates }
function Stage-PostInstall    { Invoke-PostInstallStage }
function Stage-BootstrapMarker { Write-BootstrapMarker }
function Stage-Onboard        { Invoke-OnboardWizard }

function Get-InstallStage {
    param([string]$Name)
    foreach ($s in $InstallStages) {
        if ($s.Name -eq $Name) { return $s }
    }
    return $null
}

function Invoke-Stage {
    param([Parameter(Mandatory = $true)][hashtable]$StageDef)

    Sync-EnvPath
    $script:_StageSkippedReason = $null

    $start = [DateTime]::UtcNow
    $result = @{
        stage       = $StageDef.Name
        ok          = $false
        skipped     = $false
        reason      = $null
        duration_ms = 0
    }

    try {
        & $StageDef.Worker
        $result.ok = $true
        if ($script:_StageSkippedReason) {
            $result.skipped = $true
            $result.reason = $script:_StageSkippedReason
        }
    } catch {
        $result.reason = $_.Exception.Message
        throw
    } finally {
        $result.duration_ms = [int](([DateTime]::UtcNow - $start).TotalMilliseconds)
    }

    return $result
}

function Emit-Manifest {
    $stages = foreach ($s in $InstallStages) {
        @{
            name             = $s.Name
            title            = $s.Title
            category         = $s.Category
            needs_user_input = $s.NeedsUserInput
        }
    }
    @{
        protocol_version = $InstallStageProtocolVersion
        stages           = $stages
    } | ConvertTo-Json -Depth 5 -Compress | Write-Output
}

function Resolve-DevInstallDir {
    if (-not $Dev) { return }

    $candidates = @((Get-Location).Path)
    if ($PSScriptRoot) {
        $candidates += (Split-Path -Parent $PSScriptRoot)
    }
    foreach ($path in $candidates) {
        if ($path -and (Test-Path (Join-Path $path "Cargo.toml"))) {
            $script:InstallDir = $path
            return
        }
    }
}

function Main {
    $null = Import-OrchestrateurInstallLibs
    Ensure-OrchestrateurHomeDirs
    Resolve-DevInstallDir

    Write-Banner

    if ($Dev) {
        Write-Info "Mode dev - compilation depuis les sources"
    } else {
        Write-Info "Mode release - binaire pre-compile si disponible, sinon build"
    }

    foreach ($stageDef in $InstallStages) {
        if ($stageDef.NeedsUserInput -and $NonInteractive) {
            Write-Info "Skipping $($stageDef.Name) (non-interactive)"
            continue
        }
        Write-Info $stageDef.Title
        try {
            $null = Invoke-Stage -StageDef $stageDef
        } catch {
            Write-Err "Stage $($stageDef.Name) failed: $($_.Exception.Message)"
            throw
        }
    }

    if ($Json) {
        @{ ok = $true; protocol_version = $InstallStageProtocolVersion } | ConvertTo-Json -Compress | Write-Output
    } else {
        Write-Completion
    }
}

# ============================================================================
# Entry point dispatch
# ============================================================================

if ($ProtocolVersion) {
    Write-Output $InstallStageProtocolVersion
    exit 0
}

if ($Manifest) {
    Emit-Manifest
    exit 0
}

if ($Stage) {
    $null = Import-OrchestrateurInstallLibs
    $stageDef = Get-InstallStage -Name $Stage
    if (-not $stageDef) {
        Write-Error "Unknown stage: $Stage"
        exit 2
    }
    try {
        $result = Invoke-Stage -StageDef $stageDef
        $result | ConvertTo-Json -Compress | Write-Output
        exit 0
    } catch {
        @{
            stage       = $Stage
            ok          = $false
            skipped     = $false
            reason      = $_.Exception.Message
            duration_ms = 0
        } | ConvertTo-Json -Compress | Write-Output
        exit 1
    }
}

Main