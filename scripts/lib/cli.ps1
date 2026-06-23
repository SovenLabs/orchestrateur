# Helpers CLI harness — binaire unique orch.exe ; shims orchestrateur / orchestre (style grok.exe).
$script:OrchestrateurCliBin = "orch"
$script:OrchestrateurCliClapAliases = @("orchestre", "orchestrateur")
$script:OrchestrateurCliLegacyDirs = @(
    { Join-Path $env:USERPROFILE ".cargo\bin" }
)

function Get-OrchestrateurUserBinDir {
    return Join-Path $env:USERPROFILE ".orchestrateur\bin"
}

function Write-OrchestrateurDevRepoMarker {
    param([Parameter(Mandatory = $true)][string]$Root)
    if (-not (Test-Path -LiteralPath (Join-Path $Root "Cargo.toml"))) {
        return
    }
    $stateDir = Join-Path $env:USERPROFILE ".orchestrateur"
    if (-not (Test-Path $stateDir)) {
        New-Item -ItemType Directory -Path $stateDir -Force | Out-Null
    }
    $marker = Join-Path $stateDir "dev-repo.txt"
    $utf8NoBom = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllText($marker, $Root, $utf8NoBom)
    Write-Host "  marqueur dev : $marker"
}

function Initialize-OrchestrateurBuildEnv {
    $env:Path = "$(Get-OrchestrateurUserBinDir);$env:USERPROFILE\.cargo\bin;C:\Program Files\nodejs;" + $env:Path
    $vcvars = @(
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
    ) | Where-Object { Test-Path $_ } | Select-Object -First 1
    if ($vcvars) {
        cmd /c "`"$vcvars`" && set" | ForEach-Object {
            if ($_ -match '^(.*?)=(.*)$') { Set-Item -Path "env:$($matches[1])" -Value $matches[2] }
        }
    }
}

function Get-OrchestrateurCliExe {
    param(
        [Parameter(Mandatory = $true)][string]$Root,
        [ValidateSet("release", "debug")][string]$Profile = "release"
    )
    return Join-Path $Root "target\$Profile\$($script:OrchestrateurCliBin).exe"
}

function Test-OrchestrateurCliBuilt {
    param(
        [Parameter(Mandatory = $true)][string]$Root,
        [ValidateSet("release", "debug")][string]$Profile = "release"
    )
    return Test-Path (Get-OrchestrateurCliExe -Root $Root -Profile $Profile)
}

function Remove-OrchestrateurLegacyCliArtifacts {
    param(
        [Parameter(Mandatory = $true)][string]$Root,
        [ValidateSet("release", "debug")][string]$Profile = "release"
    )
    $targetDir = Join-Path $Root "target\$Profile"
    foreach ($legacy in $script:OrchestrateurCliClapAliases) {
        $path = Join-Path $targetDir "$legacy.exe"
        if (-not (Test-Path -LiteralPath $path)) {
            continue
        }
        try {
            Remove-Item -LiteralPath $path -Force -ErrorAction Stop
            Write-Host "  artefact legacy supprime : $path"
        } catch {
            Write-Warning "Impossible de supprimer $path (processus actif ?). Arretez : Stop-Process -Name $legacy -Force"
        }
    }
}

function Build-OrchestrateurCli {
    param(
        [Parameter(Mandatory = $true)][string]$Root = "",
        [ValidateSet("release", "debug")][string]$Profile = "release",
        [string[]]$ExtraFeatures = @()
    )
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw @"
Rust/cargo introuvable dans le PATH.
Installez Rust : https://rustup.rs/
Puis relancez l'installation (ou ouvrez un nouveau terminal).
"@
    }
    $features = @("gateway", "websocket-server") + $ExtraFeatures | Select-Object -Unique
    $args = @(
        "build",
        "-p", "orchestrateur-cli",
        "--bin", $script:OrchestrateurCliBin,
        "--features", ($features -join ",")
    )
    if ($Profile -eq "release") { $args += "--release" }
    Write-Host "Compilation $Profile ($($script:OrchestrateurCliBin).exe)..."
    $previousRustflags = $env:RUSTFLAGS
    $env:RUSTFLAGS = "-D warnings"
    try {
        & cargo @args
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    } finally {
        if ($null -eq $previousRustflags) {
            Remove-Item env:RUSTFLAGS -ErrorAction SilentlyContinue
        } else {
            $env:RUSTFLAGS = $previousRustflags
        }
    }
    if ([string]::IsNullOrWhiteSpace($Root)) {
        $Root = (Get-Location).Path
    }
    Remove-OrchestrateurLegacyCliArtifacts -Root $Root -Profile $Profile
}

function Test-OrchestrateurIsAdministrator {
    $principal = New-Object Security.Principal.WindowsPrincipal(
        [Security.Principal.WindowsIdentity]::GetCurrent()
    )
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Write-OrchestrateurCliShims {
    param(
        [Parameter(Mandatory = $true)][string]$BinDir
    )
    $orchExe = Join-Path $BinDir "$($script:OrchestrateurCliBin).exe"
    if (-not (Test-Path -LiteralPath $orchExe)) {
        throw "Binaire manquant pour shims : $orchExe"
    }
    $names = @($script:OrchestrateurCliBin) + $script:OrchestrateurCliClapAliases
    foreach ($name in $names) {
        $cmdPath = Join-Path $BinDir "$name.cmd"
        @"
@echo off
"%~dp0$($script:OrchestrateurCliBin).exe" %*
"@ | Set-Content -Path $cmdPath -Encoding ASCII
        if ($name -eq $script:OrchestrateurCliBin) {
            continue
        }
        $aliasExe = Join-Path $BinDir "$name.exe"
        if (Test-Path -LiteralPath $aliasExe) {
            Remove-Item -LiteralPath $aliasExe -Force
        }
        try {
            New-Item -ItemType HardLink -Path $aliasExe -Target $orchExe -ErrorAction Stop | Out-Null
        } catch {
            Copy-Item -LiteralPath $orchExe -Destination $aliasExe -Force
        }
        Write-Host "  shim : $name.exe / $name.cmd"
    }
}

function Copy-OrchestrateurCliBinary {
    param(
        [Parameter(Mandatory = $true)][string]$Root,
        [Parameter(Mandatory = $true)][string]$DestDir,
        [ValidateSet("release", "debug")][string]$Profile = "release"
    )
    if (-not (Test-Path $DestDir)) {
        New-Item -ItemType Directory -Path $DestDir -Force | Out-Null
    }
    $src = Get-OrchestrateurCliExe -Root $Root -Profile $Profile
    if (-not (Test-Path $src)) {
        throw "Binaire manquant : $src (build echoue ?)"
    }
    $dst = Join-Path $DestDir "$($script:OrchestrateurCliBin).exe"
    $staging = "$dst.new"
    Copy-Item -Force $src $staging
    if (Test-Path -LiteralPath $dst) {
        Remove-Item -LiteralPath $dst -Force -ErrorAction Stop
    }
    Move-Item -LiteralPath $staging -Destination $dst -Force
    Write-Host "  -> $dst"
    Write-OrchestrateurCliShims -BinDir $DestDir
}

function Remove-OrchestrateurCliFromLegacyDirs {
    $names = @($script:OrchestrateurCliBin) + $script:OrchestrateurCliClapAliases
    foreach ($dirExpr in $script:OrchestrateurCliLegacyDirs) {
        $dir = & $dirExpr
        if (-not (Test-Path -LiteralPath $dir)) { continue }
        foreach ($name in $names) {
            foreach ($ext in @("exe", "cmd")) {
                $path = Join-Path $dir "$name.$ext"
                if (-not (Test-Path -LiteralPath $path)) { continue }
                try {
                    Remove-Item -LiteralPath $path -Force -ErrorAction Stop
                    Write-Host "  ancien binaire retire : $path"
                } catch {
                    Write-Warning "Impossible de supprimer $path (processus actif ?)"
                }
            }
        }
    }
}

function Add-OrchestrateurPathEntry {
    param(
        [Parameter(Mandatory = $true)][string]$Dir,
        [ValidateSet("User", "Machine")][string]$Scope = "User"
    )
    $current = [Environment]::GetEnvironmentVariable("Path", $Scope)
    if ($current -like "*$Dir*") {
        Write-Host "PATH $Scope : $Dir deja present"
        return
    }
    $newPath = if ([string]::IsNullOrWhiteSpace($current)) { $Dir } else { "$Dir;$current" }
    [Environment]::SetEnvironmentVariable("Path", $newPath, $Scope)
    $env:Path = "$Dir;" + ($env:Path -replace [regex]::Escape("$Dir;"), "" -replace [regex]::Escape(";$Dir"), "")
    Write-Host "PATH $Scope mis a jour : $Dir"
}

function Install-OrchestrateurCliToUserPath {
    param(
        [Parameter(Mandatory = $true)][string]$Root,
        [ValidateSet("release", "debug")][string]$Profile = "release"
    )
    $binDir = Get-OrchestrateurUserBinDir
    Copy-OrchestrateurCliBinary -Root $Root -DestDir $binDir -Profile $Profile
    Write-OrchestrateurDevRepoMarker -Root $Root
    Remove-OrchestrateurCliFromLegacyDirs
    Add-OrchestrateurPathEntry -Dir $binDir -Scope "User"
}

function Install-OrchestrateurCliToMachinePath {
    param(
        [Parameter(Mandatory = $true)][string]$Root,
        [ValidateSet("release", "debug")][string]$Profile = "release"
    )
    if (-not (Test-OrchestrateurIsAdministrator)) {
        throw @"
Installation systeme (-AllUsers) : lancez PowerShell en administrateur.
  Exemple : .\install.ps1 -Dev -SkipBuild -AllUsers
"@
    }
    $installDir = Join-Path ${env:ProgramFiles} "Orchestrateur"
    Copy-OrchestrateurCliBinary -Root $Root -DestDir $installDir -Profile $Profile
    Add-OrchestrateurPathEntry -Dir $installDir -Scope "Machine"
    Write-Host "CMD administrateur : orchestrateur disponible apres ouverture d'un nouveau terminal."
}

function Write-OrchestrateurCliUsageHint {
    param([Parameter(Mandatory = $true)][string]$Workspace)
    Write-Host ""
    Write-Host "Commandes (meme binaire) : orchestrateur, orchestre, orch"
    Write-Host "  orchestrateur doctor --workspace `"$Workspace`""
    Write-Host "  orchestrateur daemon run --workspace `"$Workspace`""
}