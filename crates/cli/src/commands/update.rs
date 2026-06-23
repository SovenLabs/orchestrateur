//! `orch update` — mise à jour automatique (release ou dev).

use anyhow::Result;
use clap::Args;
use orchestrator::VERSION;
use serde::Deserialize;

use orchestrator::stop_daemon;
use crate::windows_ops::{find_dev_repo_root, powershell_install_body, spawn_detached_after_exit};

const REPO: &str = "SovenLabs/orchestrateur";

/// Options de mise à jour.
#[derive(Debug, Clone, Args, Default)]
pub struct UpdateArgs {
    /// Vérifie seulement si une mise à jour est disponible (exit 0 = à jour).
    #[arg(long)]
    pub check: bool,
    /// Force la recompilation depuis le dépôt local.
    #[arg(long)]
    pub dev: bool,
    /// Force l'installateur release GitHub (sans détection auto).
    #[arg(long, hide = true)]
    pub release: bool,
}

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
}

async fn fetch_latest_release_version() -> Result<Option<String>> {
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

fn resolve_mode(args: &UpdateArgs) -> bool {
    if args.dev {
        return true;
    }
    if args.release {
        return false;
    }
    find_dev_repo_root().is_some()
}

/// Met à jour Orchestrateur sans interaction (release GitHub ou dev local selon contexte).
pub async fn run(args: UpdateArgs) -> Result<()> {
    let latest = fetch_latest_release_version().await?;
    let dev_mode = resolve_mode(&args);

    println!("Version locale : v{VERSION}");
    match &latest {
        Some(v) => println!("Dernière release : v{v}"),
        None => println!("Dernière release : (aucune publiée sur GitHub)"),
    }

    if args.check {
        return handle_check_only(&latest, dev_mode);
    }

    if latest.as_deref() == Some(VERSION) && !args.dev {
        println!("Déjà à jour (v{VERSION}). Utilisez --dev pour recompiler.");
        return Ok(());
    }

    println!("Arrêt du daemon…");
    stop_daemon().map_err(|e| anyhow::anyhow!("{e}"))?;

    if dev_mode {
        if let Some(root) = find_dev_repo_root() {
            println!("Mode : dev ({})", root.display());
        } else {
            println!("Mode : dev (compile → ~/.orchestrateur/bin)");
        }
    } else {
        println!("Mode : release (GitHub)");
    }

    let repo = if dev_mode { find_dev_repo_root() } else { None };
    let task = powershell_install_body(dev_mode, repo.as_deref())?;
    spawn_detached_after_exit(&task)?;

    println!();
    println!("Mise à jour lancée en arrière-plan.");
    println!("Rouvrez le terminal puis : orch --version");
    std::process::exit(0);
}

fn handle_check_only(latest: &Option<String>, dev_mode: bool) -> Result<()> {
    match latest {
        Some(v) if VERSION == *v => {
            println!("À jour.");
            std::process::exit(0);
        }
        Some(_) => {
            println!("Mise à jour disponible — lancez `orch update`.");
            std::process::exit(1);
        }
        None if dev_mode => {
            println!("Pas de release GitHub — mode dev disponible (`orch update`).");
            std::process::exit(0);
        }
        None => {
            println!("Pas de release GitHub publiée.");
            std::process::exit(1);
        }
    }
}