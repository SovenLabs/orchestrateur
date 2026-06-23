//! `orch daemon` — gestion du daemon WebSocket.

use std::path::Path;

use anyhow::{Context, Result};
use clap::Subcommand;

use crate::present;

/// Sous-commandes daemon.
#[derive(Debug, Clone, Subcommand)]
pub enum DaemonCommands {
    /// Démarre le daemon WS (bloquant, Ctrl+C pour arrêter).
    Start {
        #[arg(long)]
        port: Option<u16>,
        #[arg(long)]
        bind: Option<String>,
    },
    Stop,
    Status,
    Restart,
    Install,
    #[command(hide = true)]
    Run {
        #[arg(long)]
        port: Option<u16>,
        #[arg(long)]
        bind: Option<String>,
    },
}

pub async fn run(cmd: DaemonCommands, workspace: &Path) -> Result<Option<()>> {
    match cmd {
        DaemonCommands::Install => {
            present::daemon_install(workspace)?;
            Ok(Some(()))
        }
        DaemonCommands::Status => {
            present::daemon_status(workspace).await?;
            Ok(Some(()))
        }
        DaemonCommands::Stop => {
            present::daemon_stop()?;
            Ok(Some(()))
        }
        DaemonCommands::Restart => {
            present::daemon_stop()?;
            spawn_daemon_background(workspace)?;
            println!("Daemon redémarré en arrière-plan.");
            Ok(Some(()))
        }
        DaemonCommands::Start { .. } | DaemonCommands::Run { .. } => Ok(None),
    }
}

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