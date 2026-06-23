//! Mise à jour autonome du binaire Orchestrateur (release GitHub ou dev local).

use std::path::{Path, PathBuf};
use std::process::{Command as OsCommand, Stdio};

use anyhow::{Context, Result};
use orchestrator::VERSION;
use serde::Deserialize;

use crate::harness_ops::daemon_stop;

const REPO: &str = "SovenLabs/orchestrateur";
const INSTALL_PS1_URL: &str =
    "https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.ps1";

/// Options de la commande `orchestrateur update`.
#[derive(Debug, Clone, Default)]
pub struct UpdateOptions {
    /// Compare uniquement la version locale vs GitHub (exit 0 = à jour).
    pub check: bool,
    /// Force la mise à jour dev (`install.ps1 -Dev`).
    pub dev: bool,
    /// Force la mise à jour release (Setup GitHub).
    pub release: bool,
}

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
}

/// Récupère la version de la dernière release GitHub (`v0.28.0` → `0.28.0`).
pub async fn fetch_latest_release_version() -> Result<Option<String>> {
    let client = reqwest::Client::builder()
        .user_agent("Orchestrateur-CLI-Update")
        .build()?;
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let response = client.get(&url).send().await?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let release: GhRelease = response.error_for_status()?.json().await?;
    Ok(Some(normalize_tag(&release.tag_name)))
}

fn normalize_tag(tag: &str) -> String {
    let t = tag.trim();
    if let Some(stripped) = t.strip_prefix('v') {
        stripped.to_string()
    } else {
        t.to_string()
    }
}

fn find_dev_repo_root() -> Option<PathBuf> {
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
        for _ in 0..10 {
            let cargo = dir.join("Cargo.toml");
            let install = dir.join("install.ps1");
            if cargo.is_file() && install.is_file() {
                return Some(dir);
            }
            dir = dir.parent()?.to_path_buf();
        }
    }
    None
}

fn resolve_update_mode(opts: &UpdateOptions) -> Result<bool> {
    if opts.dev {
        return Ok(true);
    }
    if opts.release {
        return Ok(false);
    }
    Ok(find_dev_repo_root().is_some())
}

async fn download_install_script() -> Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("Orchestrateur-CLI-Update")
        .build()?;
    Ok(client
        .get(INSTALL_PS1_URL)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?)
}

#[cfg(windows)]
async fn run_powershell_install(dev: bool, repo_root: Option<&Path>) -> Result<()> {
    if dev {
        let discovered = find_dev_repo_root();
        let root = repo_root
            .or(discovered.as_deref())
            .context("mode dev : clone orchestrateur introuvable (Cargo.toml + install.ps1)")?;
        let script = root.join("install.ps1");
        println!("Mise à jour dev depuis {} …", root.display());
        let status = OsCommand::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                &script.to_string_lossy(),
                "-Dev",
                "-SkipDoctor",
            ])
            .current_dir(root)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("powershell install.ps1 -Dev")?;
        if !status.success() {
            anyhow::bail!("install.ps1 a échoué (code {status})");
        }
        return Ok(());
    }

    let temp_dir = std::env::temp_dir().join("orchestrateur-update");
    std::fs::create_dir_all(&temp_dir).context("mkdir temp update")?;
    let script_path = temp_dir.join("install.ps1");
    println!("Téléchargement installateur release…");
    std::fs::write(&script_path, download_install_script().await?).context("écriture install.ps1 temp")?;

    println!("Installation release en cours…");
    let status = OsCommand::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script_path.to_string_lossy(),
            "-SkipDoctor",
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("powershell install.ps1 release")?;
    if !status.success() {
        anyhow::bail!("install.ps1 release a échoué (code {status})");
    }
    Ok(())
}

#[cfg(not(windows))]
async fn run_powershell_install(_dev: bool, _repo_root: Option<&Path>) -> Result<()> {
    anyhow::bail!(
        "orchestrateur update automatique : Windows uniquement pour l'instant.\n\
         Linux/macOS : cargo build --release -p orchestrateur-cli --bin orch"
    );
}

/// Met à jour Orchestrateur (release ou dev selon détection / flags).
pub async fn cmd_update(opts: UpdateOptions) -> Result<()> {
    let latest = fetch_latest_release_version().await?;
    let dev_mode = resolve_update_mode(&opts)?;

    println!("Version locale : v{VERSION}");
    match &latest {
        Some(v) => println!("Dernière release : v{v}"),
        None => println!("Dernière release : (aucune release publiée sur GitHub)"),
    }

    if opts.check {
        match &latest {
            Some(v) if VERSION == *v => {
                println!("À jour.");
                std::process::exit(0);
            }
            Some(_) => {
                println!("Mise à jour disponible.");
                std::process::exit(1);
            }
            None if dev_mode => {
                println!("Pas de release GitHub — mode dev détecté. Utilisez `orchestrateur update --dev` pour recompiler.");
                std::process::exit(0);
            }
            None => {
                println!("Pas de release GitHub publiée.");
                std::process::exit(1);
            }
        }
    }

    if latest.as_deref() == Some(VERSION) && !opts.dev {
        println!("Déjà à jour (v{VERSION}). Utilisez --dev pour recompiler depuis le dépôt.");
        return Ok(());
    }

    println!("Arrêt du daemon avant mise à jour…");
    daemon_stop()?;

    let repo = if dev_mode {
        find_dev_repo_root()
    } else {
        None
    };

    if dev_mode {
        println!("Mode : développement (compile + ~/.orchestrateur/bin)");
    } else {
        println!("Mode : release (GitHub Setup)");
    }

    run_powershell_install(dev_mode, repo.as_deref()).await?;

    println!();
    println!("Mise à jour terminée.");
    println!("Fermez et rouvrez le terminal, puis : orchestrateur --version");
    Ok(())
}