//! Cycle de vie des agents persistants (heartbeat).

mod background;
mod sleep;
mod wake;

pub use background::{run_background_tasks, BackgroundTaskReport};
pub use sleep::sleep_agent;
pub use wake::wake_agent;

use std::path::Path;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::persistent::{AgentStatus, PersistentAgentError};

/// Configuration du heartbeat lue/écrite dans `heartbeat.md`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentHeartbeat {
    /// Intervalle recommandé entre cycles (secondes).
    pub interval_secs: u64,
    /// Dernier réveil ISO-8601.
    pub last_wake: Option<String>,
    /// Dernière mise en veille ISO-8601.
    pub last_sleep: Option<String>,
    /// Tâches exécutées en mode background.
    pub background_tasks: Vec<String>,
}

impl Default for AgentHeartbeat {
    fn default() -> Self {
        Self {
            interval_secs: 300,
            last_wake: None,
            last_sleep: None,
            background_tasks: vec![
                "check_inbox".into(),
                "scan_tasks".into(),
            ],
        }
    }
}

impl AgentHeartbeat {
    /// Charge `heartbeat.md` depuis le dossier agent (valeurs par défaut si absent).
    pub async fn load(agent_root: &Path) -> Result<Self, PersistentAgentError> {
        let path = agent_root.join("heartbeat.md");
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(&path).await.map_err(|e| {
            PersistentAgentError::Io(format!("lecture {}: {e}", path.display()))
        })?;
        Ok(parse_heartbeat_md(&raw))
    }

    /// Persiste le heartbeat dans `heartbeat.md`.
    pub async fn save(&self, agent_root: &Path) -> Result<(), PersistentAgentError> {
        let path = agent_root.join("heartbeat.md");
        fs::write(&path, render_heartbeat_md(self))
            .await
            .map_err(|e| PersistentAgentError::Io(e.to_string()))
    }

    /// Marque un réveil.
    pub fn mark_wake(&mut self) {
        self.last_wake = Some(Utc::now().to_rfc3339());
    }

    /// Marque une mise en veille.
    pub fn mark_sleep(&mut self) {
        self.last_sleep = Some(Utc::now().to_rfc3339());
    }
}

fn parse_heartbeat_md(raw: &str) -> AgentHeartbeat {
    let mut hb = AgentHeartbeat::default();
    let mut in_tasks = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("interval_secs:") {
            if let Some(v) = trimmed.split(':').nth(1) {
                if let Ok(n) = v.trim().parse() {
                    hb.interval_secs = n;
                }
            }
        } else if trimmed.starts_with("last_wake:") {
            let v = trimmed.split(':').skip(1).collect::<Vec<_>>().join(":").trim().to_string();
            hb.last_wake = if v == "null" || v.is_empty() { None } else { Some(v) };
        } else if trimmed.starts_with("last_sleep:") {
            let v = trimmed.split(':').skip(1).collect::<Vec<_>>().join(":").trim().to_string();
            hb.last_sleep = if v == "null" || v.is_empty() { None } else { Some(v) };
        } else if trimmed == "background_tasks:" {
            in_tasks = true;
        } else if in_tasks && trimmed.starts_with("- ") {
            hb.background_tasks.push(trimmed[2..].trim().to_string());
        }
    }
    hb
}

fn render_heartbeat_md(hb: &AgentHeartbeat) -> String {
    let wake = hb.last_wake.as_deref().unwrap_or("null");
    let sleep = hb.last_sleep.as_deref().unwrap_or("null");
    let tasks = hb
        .background_tasks
        .iter()
        .map(|t| format!("  - {t}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "# Heartbeat\n\n\
         interval_secs: {}\n\
         last_wake: {wake}\n\
         last_sleep: {sleep}\n\
         background_tasks:\n{tasks}\n",
        hb.interval_secs
    )
}

/// Met à jour le statut et persiste config + heartbeat.
pub(crate) async fn persist_lifecycle(
    agent_root: &std::path::Path,
    config: &mut crate::persistent::PersistentAgentConfig,
    status: AgentStatus,
    heartbeat: &mut AgentHeartbeat,
) -> Result<(), PersistentAgentError> {
    config.status = status;
    config.last_heartbeat = Some(Utc::now().to_rfc3339());
    crate::persistent::AgentStructure::write_config(agent_root, config).await?;
    heartbeat.save(agent_root).await
}