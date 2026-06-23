//! Mise à jour autonome du binaire Orchestrateur (release GitHub ou dev local).

use anyhow::Result;
use orchestrator::VERSION;
use serde::Deserialize;

use crate::harness_ops::daemon_stop;
use crate::windows_ops::{find_dev_repo_root, powershell_install_body, spawn_detached_after_exit};

const REPO: &str = "SovenLabs/orchestrateur";

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

fn resolve_update_mode(opts: &UpdateOptions) -> Result<bool> {
    if opts.dev {
        return Ok(true);
    }
    if opts.release {
        return Ok(false);
    }
    Ok(find_dev_repo_root().is_some())
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
                println!("Pas de release GitHub — mode dev détecté. Lancez `orchestrateur update` pour recompiler.");
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
        let dev_root = repo.clone().or_else(find_dev_repo_root);
        if let Some(root) = dev_root {
            println!("Mode : développement depuis {}", root.display());
        } else {
            println!("Mode : développement (compile + ~/.orchestrateur/bin)");
        }
    } else {
        println!("Mode : release (GitHub Setup)");
    }

    let task = powershell_install_body(dev_mode, repo.as_deref())?;
    spawn_detached_after_exit(&task)?;

    println!();
    println!("Mise à jour lancée en arrière-plan (le binaire en cours sera remplacé).");
    println!("Une fenêtre PowerShell va terminer l'installation.");
    println!("Puis rouvrez le terminal : orchestrateur --version");
    std::process::exit(0);
}