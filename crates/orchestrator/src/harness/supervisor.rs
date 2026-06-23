//! Superviseur harness — démarre daemon/gateway si absents.

use std::path::Path;
use std::process::{Command as OsCommand, Stdio};
use std::time::Duration;

use crate::config::OrchestratorConfig;
use crate::harness::client::probe_client;
use crate::harness::env::ensure_daemon_token;
use crate::harness::error::HarnessError;

/// Services démarrés par le superviseur (PIDs gérés par le CLI).
#[derive(Debug, Clone, Default)]
pub struct SupervisorSpawnPlan {
    /// Lancer daemon start.
    pub spawn_daemon: bool,
    /// Lancer gateway run.
    pub spawn_gateway: bool,
    /// URL daemon.
    pub daemon_url: String,
    /// URL gateway.
    pub gateway_url: String,
}

/// Détermine quels services doivent être démarrés.
pub async fn plan_supervisor(workspace: &Path) -> Result<SupervisorSpawnPlan, HarnessError> {
    let config = OrchestratorConfig::load_workspace(workspace)?;
    ensure_daemon_token()?;

    let daemon_url = format!("http://{}:{}/health", config.daemon.bind, config.daemon.port);
    let gateway_url = format!("http://{}:{}/health", config.gateway.bind, config.gateway.port);

    let spawn_daemon = config.daemon.enabled && !service_alive(&daemon_url).await;
    let spawn_gateway = config.gateway.enabled && !service_alive(&gateway_url).await;

    Ok(SupervisorSpawnPlan {
        spawn_daemon,
        spawn_gateway,
        daemon_url,
        gateway_url,
    })
}

/// Spawn processus enfant CLI (daemon start / gateway run).
pub fn spawn_child(args: &[&str]) -> Result<std::process::Child, HarnessError> {
    let exe = std::env::current_exe().map_err(|e| HarnessError::Platform(e.to_string()))?;
    OsCommand::new(&exe)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| HarnessError::Platform(e.to_string()))
}

/// Attente courte après spawn.
pub async fn wait_for_spawn() {
    tokio::time::sleep(Duration::from_secs(2)).await;
}

async fn service_alive(url: &str) -> bool {
    probe_client()
        .get(url)
        .send()
        .await
        .map_or(false, |r| r.status().is_success())
}