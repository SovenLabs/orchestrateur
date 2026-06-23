//! Chargement des agents persistants depuis le disque.

use std::path::Path;

use tokio::fs;

use crate::persistent::{AgentIdentity, AgentStructure, PersistentAgent, PersistentAgentError};

/// Charge tous les agents valides depuis `workspace/agents/`.
///
/// # Errors
///
/// Propage [`PersistentAgentError`] si un dossier agent est illisible.
pub async fn load_agents(agents_dir: &Path) -> Result<Vec<PersistentAgent>, PersistentAgentError> {
    if !agents_dir.exists() {
        return Ok(Vec::new());
    }

    let mut agents = Vec::new();
    let mut entries = fs::read_dir(agents_dir).await.map_err(|e| {
        PersistentAgentError::Io(format!("listage {}: {e}", agents_dir.display()))
    })?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| {
        PersistentAgentError::Io(e.to_string())
    })? {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(id) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !path.join("config.toml").exists() {
            continue;
        }
        match AgentStructure::load_agent(agents_dir, id).await {
            Ok(agent) => agents.push(agent),
            Err(e) => tracing::warn!(agent = %id, %e, "agent ignoré au chargement"),
        }
    }

    agents.sort_by(|a, b| a.id().cmp(b.id()));
    Ok(agents)
}