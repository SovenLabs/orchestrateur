//! `orch daemon` — gestion du daemon WebSocket.

use std::path::Path;

use anyhow::{Context, Result};
use clap::Subcommand;

use crate::harness_ops::{daemon_install, daemon_status, daemon_stop};

/// Sous-commandes daemon.
#[derive(Debug, Clone, Subcommand)]
pub enum DaemonCommands {
    /// Démarre le daemon WS (bloquant, Ctrl+C pour arrêter).
    Start {
        /// Port (surcharge orchestrator.toml).
        #[arg(long)]
        port: Option<u16>,
        /// Adresse de liaison.
        #[arg(long)]
        bind: Option<String>,
    },
    /// Arrête le daemon (processus + tâche planifiée).
    Stop,
    /// Affiche le statut (tâche Windows + sonde /health).
    Status,
    /// Redémarre le daemon (stop puis start en arrière-plan).
    Restart,
    /// Installe la tâche planifiée Windows (démarrage à la connexion).
    Install,
    /// Alias historique de `start`.
    #[command(hide = true)]
    Run {
        #[arg(long)]
        port: Option<u16>,
        #[arg(long)]
        bind: Option<String>,
    },
}

/// Dispatch des commandes daemon (hors `start`/`run` bloquants).
pub async fn run(cmd: DaemonCommands, workspace: &Path) -> Result<Option<()>> {
    match cmd {
        DaemonCommands::Install => {
            daemon_install(workspace)?;
            Ok(Some(()))
        }
        DaemonCommands::Status => {
            daemon_status(workspace).await?;
            Ok(Some(()))
        }
        DaemonCommands::Stop => {
            daemon_stop()?;
            Ok(Some(()))
        }
        DaemonCommands::Restart => {
            daemon_stop()?;
            spawn_daemon_background(workspace)?;
            println!("Daemon redémarré en arrière-plan.");
            Ok(Some(()))
        }
        DaemonCommands::Start { .. } | DaemonCommands::Run { .. } => Ok(None),
    }
}

/// Démarre le daemon en processus détaché (pour restart / harness).
pub fn spawn_daemon_background(workspace: &Path) -> Result<()> {
    let exe = std::env::current_exe().context("binaire CLI")?;
    let ws = workspace.to_string_lossy().to_string();
    std::process::Command::new(&exe)
        .args(["daemon", "start", "--workspace", &ws])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("spawn daemon start")?;
    Ok(())
}