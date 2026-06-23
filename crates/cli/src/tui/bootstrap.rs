//! Attente du démarrage daemon/gateway après onboard (barre de progression).

use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use orchestrator::OrchestratorConfig;
use tokio::time::sleep;

use crate::harness_ops::{probe_daemon_health, probe_gateway_health, ServiceProbeState};

use super::progress::ProgressSession;

/// Attend que daemon et gateway répondent (sonde HTTP), avec barre de progression.
pub async fn wait_for_harness(workspace: &Path) -> Result<()> {
    let config = OrchestratorConfig::load_workspace(workspace)
        .map_err(|e| anyhow::anyhow!("config: {e}"))?;

    let mut progress = ProgressSession::new("Démarrage Orchestrateur");
    progress.tick("Vérification de la configuration…", 12);

    let daemon_url = format!(
        "http://{}:{}/health",
        config.daemon.bind, config.daemon.port
    );
    let gateway_url = format!(
        "http://{}:{}/health",
        config.gateway.bind, config.gateway.port
    );

    progress.tick("Token daemon et gateway…", 24);

    let mut daemon = ServiceProbeState::Unknown;
    for attempt in 0..20 {
        let pct = 24 + (attempt + 1) * 2;
        progress.set(
            format!("Attente du daemon ({daemon_url})…"),
            pct.min(55),
        );
        daemon = probe_daemon_health(&daemon_url).await;
        if daemon.is_alive() {
            break;
        }
        sleep(Duration::from_millis(750)).await;
    }

    let mut gateway = ServiceProbeState::Unknown;
    for attempt in 0..20 {
        let pct = 56 + (attempt + 1) * 2;
        progress.set(
            format!("Attente du gateway messaging ({gateway_url})…"),
            pct.min(92),
        );
        gateway = probe_gateway_health(&gateway_url).await;
        if gateway.is_alive() {
            break;
        }
        sleep(Duration::from_millis(750)).await;
    }

    let ok = daemon.is_alive() && gateway.is_alive();
    let summary = if ok {
        "Daemon et gateway actifs".to_string()
    } else if daemon.is_alive() {
        "Daemon actif — gateway arrêté (lancez : orchestrateur harness run)".to_string()
    } else {
        "Services arrêtés — lancez : orchestrateur harness run".to_string()
    };
    progress.finish(summary, ok)?;

    if !daemon.is_alive() {
        println!("  Daemon : arrêté ou injoignable");
        println!("  → orchestrateur daemon run --workspace \"{}\"", workspace.display());
    } else {
        println!("  Daemon : actif");
    }
    if !gateway.is_alive() {
        println!("  Gateway : arrêté ou injoignable");
        println!("  → orchestrateur harness run --workspace \"{}\"", workspace.display());
    } else {
        println!("  Gateway : actif");
    }

    let _ = (daemon, gateway);
    Ok(())
}