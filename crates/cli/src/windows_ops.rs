//! Opérations Windows détachées (self-update / self-uninstall sans verrouiller le binaire).

use std::path::{Path, PathBuf};
use std::process::{Command as OsCommand, Stdio};

use anyhow::{Context, Result};

const DEV_REPO_MARKER: &str = "dev-repo.txt";
const INSTALL_PS1_URL: &str =
    "https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.ps1";
const UNINSTALL_PS1_URL: &str =
    "https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/uninstall.ps1";

pub fn orchestrateur_state_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("USERPROFILE").map(PathBuf::from) {
        return home.join(".orchestrateur");
    }
    if let (Ok(drive), Ok(path)) = (std::env::var("HOMEDRIVE"), std::env::var("HOMEPATH")) {
        return PathBuf::from(format!("{drive}{path}")).join(".orchestrateur");
    }
    if let Some(home) = std::env::home_dir() {
        return home.join(".orchestrateur");
    }
    PathBuf::from(".orchestrateur")
}

/// Racine du dépôt dev : marqueur install, CWD, ou parent de l'exe.
pub fn find_dev_repo_root() -> Option<PathBuf> {
    if let Some(root) = read_dev_repo_marker() {
        return Some(root);
    }

    let mut candidates = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.to_path_buf());
        }
    }

    for mut dir in candidates {
        for _ in 0..12 {
            if is_dev_repo_root(&dir) {
                return Some(dir);
            }
            dir = dir.parent()?.to_path_buf();
        }
    }
    None
}

fn is_dev_repo_root(dir: &Path) -> bool {
    dir.join("Cargo.toml").is_file() && dir.join("install.ps1").is_file()
}

fn normalize_marker_path(raw: &str) -> String {
    raw.trim()
        .trim_start_matches('\u{FEFF}')
        .trim()
        .to_string()
}

pub fn read_dev_repo_marker() -> Option<PathBuf> {
    let marker = orchestrateur_state_dir().join(DEV_REPO_MARKER);
    let content = std::fs::read_to_string(&marker).ok()?;
    let path = PathBuf::from(normalize_marker_path(&content));
    (!path.as_os_str().is_empty()).then_some(path)
}

/// Lance un script PowerShell après la sortie du processus CLI courant.
#[cfg(windows)]
pub fn spawn_detached_after_exit(task_body: &str) -> Result<()> {
    let parent_pid = std::process::id();
    let temp_dir = std::env::temp_dir().join("orchestrateur-spawn");
    std::fs::create_dir_all(&temp_dir).context("mkdir orchestrateur-spawn")?;
    let script_path = temp_dir.join(format!("task-{parent_pid}.ps1"));
    let script = format!(
        r#"$ErrorActionPreference = 'Stop'
$parentPid = {parent_pid}
$deadline = (Get-Date).AddSeconds(60)
while ((Get-Date) -lt $deadline) {{
    if (-not (Get-Process -Id $parentPid -ErrorAction SilentlyContinue)) {{ break }}
    Start-Sleep -Milliseconds 250
}}
{task_body}
"#
    );
    std::fs::write(&script_path, script).context("écriture script détaché")?;
    OsCommand::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script_path.to_string_lossy(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("lancement PowerShell détaché")?;
    Ok(())
}

#[cfg(not(windows))]
pub fn spawn_detached_after_exit(_task_body: &str) -> Result<()> {
    anyhow::bail!("opération détachée : Windows uniquement");
}

/// Corps PowerShell : arrêt processus + install.ps1.
pub fn powershell_install_body(dev: bool, repo_root: Option<&Path>) -> Result<String> {
    if dev {
        let discovered = find_dev_repo_root();
        let root = repo_root
            .or(discovered.as_deref())
            .context("mode dev : dépôt orchestrateur introuvable (réinstallez avec install.ps1 -Dev depuis le clone)")?;
        let script = root.join("install.ps1");
        let root_esc = escape_ps_single_quoted(&root.to_string_lossy());
        let script_esc = escape_ps_single_quoted(&script.to_string_lossy());
        return Ok(format!(
            "Set-Location -LiteralPath '{root_esc}'\n\
             & powershell -NoProfile -ExecutionPolicy Bypass -File '{script_esc}' -Dev -SkipDoctor"
        ));
    }

    Ok(format!(
        "$env:ORCHESTRATEUR_SKIP_DOCTOR = '1'\n\
         Invoke-Expression (Invoke-WebRequest -UseBasicParsing -Uri '{INSTALL_PS1_URL}').Content"
    ))
}

fn escape_ps_single_quoted(s: &str) -> String {
    s.replace('\'', "''")
}

/// Corps PowerShell pour désinstallation complète.
pub fn powershell_uninstall_body() -> String {
    if let Some(root) = read_dev_repo_marker().or_else(find_dev_repo_root) {
        let script = root.join("uninstall.ps1");
        let script_esc = escape_ps_single_quoted(&script.to_string_lossy());
        return format!("& powershell -NoProfile -ExecutionPolicy Bypass -File '{script_esc}'");
    }
    format!("Invoke-Expression (Invoke-WebRequest -UseBasicParsing -Uri '{UNINSTALL_PS1_URL}').Content")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_repo_marker_is_readable() {
        let marker = orchestrateur_state_dir().join(DEV_REPO_MARKER);
        assert!(marker.is_file(), "marker missing: {}", marker.display());
        let root = read_dev_repo_marker().expect("read_dev_repo_marker");
        assert_eq!(find_dev_repo_root().as_deref(), Some(root.as_path()));
    }
}