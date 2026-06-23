//! Sondes HTTP daemon / gateway partagées CLI ↔ desktop Tauri.

use reqwest::Client;
use serde::Serialize;

use crate::config::OrchestratorConfig;

/// État d'un service HTTP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceHealth {
    /// `/health` répond 2xx.
    Alive,
    /// Service injoignable ou erreur HTTP.
    Down,
}

impl ServiceHealth {
    /// Libellé wire (`alive` / `down`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Alive => "alive",
            Self::Down => "down",
        }
    }
}

/// Résultat des sondes harness (daemon + gateway).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct HarnessServiceProbe {
    /// État daemon (`alive` / `down` / `skipped`).
    pub daemon: String,
    /// État gateway (`alive` / `down` / `skipped`).
    pub gateway: String,
    /// URL sonde daemon.
    pub daemon_url: String,
    /// URL sonde gateway.
    pub gateway_url: String,
    /// Gateway activé dans la config workspace.
    pub gateway_enabled: bool,
}

/// Sonde une URL `/health`.
pub async fn probe_health(client: &Client, url: &str) -> ServiceHealth {
    match client.get(url).send().await {
        Ok(r) if r.status().is_success() => ServiceHealth::Alive,
        _ => ServiceHealth::Down,
    }
}

/// Sondes daemon et gateway selon [`OrchestratorConfig`].
pub async fn probe_harness_services(
    config: &OrchestratorConfig,
    client: &Client,
) -> HarnessServiceProbe {
    let daemon_url = format!(
        "http://{}:{}/health",
        config.daemon.bind, config.daemon.port
    );
    let gateway_url = format!(
        "http://{}:{}/health",
        config.gateway.bind, config.gateway.port
    );

    let daemon = if config.daemon.enabled {
        probe_health(client, &daemon_url).await.as_str().to_string()
    } else {
        "skipped".into()
    };

    let gateway_enabled = config.gateway.enabled;
    let gateway = if gateway_enabled {
        probe_health(client, &gateway_url).await.as_str().to_string()
    } else {
        "skipped".into()
    };

    HarnessServiceProbe {
        daemon,
        gateway,
        daemon_url,
        gateway_url,
        gateway_enabled,
    }
}