//! Lecture des messages inter-agents (inbox).

use std::path::Path;

use tokio::fs;

use super::message::AgentMessage;
use crate::persistent::PersistentAgentError;

/// Lit les messages de l'inbox d'un agent.
///
/// Si `mark_read` est vrai, marque chaque message lu sur disque.
pub async fn receive_messages(
    agent_root: &Path,
    mark_read: bool,
) -> Result<Vec<AgentMessage>, PersistentAgentError> {
    let inbox = agent_root.join("messages").join("inbox");
    if !inbox.exists() {
        return Ok(Vec::new());
    }

    let mut messages = Vec::new();
    let mut entries = fs::read_dir(&inbox).await.map_err(io)?;
    while let Some(entry) = entries.next_entry().await.map_err(io)? {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let raw = fs::read_to_string(&path).await.map_err(io)?;
        let mut msg: AgentMessage =
            serde_json::from_str(&raw).map_err(|e| PersistentAgentError::Config(e.to_string()))?;
        if mark_read && !msg.read {
            msg.read = true;
            let updated = serde_json::to_string_pretty(&msg)
                .map_err(|e| PersistentAgentError::Config(e.to_string()))?;
            fs::write(&path, updated).await.map_err(io)?;
        }
        messages.push(msg);
    }

    messages.sort_by(|a, b| a.sent_at.cmp(&b.sent_at));
    Ok(messages)
}

fn io(e: std::io::Error) -> PersistentAgentError {
    PersistentAgentError::Io(e.to_string())
}