//! Envoi de messages inter-agents.

use std::path::Path;

use tokio::fs;

use super::message::AgentMessage;
use crate::persistent::PersistentAgentError;

/// Écrit un message dans l'outbox de l'émetteur et l'inbox du destinataire.
///
/// # Errors
///
/// Propage [`PersistentAgentError`] si les dossiers agents sont introuvables ou l'écriture échoue.
pub async fn send_message(
    agents_dir: &Path,
    from: &str,
    to: &str,
    body: &str,
) -> Result<AgentMessage, PersistentAgentError> {
    let from_root = agents_dir.join(from);
    let to_root = agents_dir.join(to);
    if !from_root.join("config.toml").exists() {
        return Err(PersistentAgentError::NotFound(from.to_string()));
    }
    if !to_root.join("config.toml").exists() {
        return Err(PersistentAgentError::NotFound(to.to_string()));
    }

    let message = AgentMessage::new(from, to, body);
    let payload = serde_json::to_string_pretty(&message)
        .map_err(|e| PersistentAgentError::Config(e.to_string()))?;
    let filename = format!("{}.json", message.id);

    let outbox = from_root.join("messages").join("outbox");
    let inbox = to_root.join("messages").join("inbox");
    fs::create_dir_all(&outbox).await.map_err(io)?;
    fs::create_dir_all(&inbox).await.map_err(io)?;

    fs::write(outbox.join(&filename), &payload).await.map_err(io)?;
    fs::write(inbox.join(&filename), payload).await.map_err(io)?;

    Ok(message)
}

fn io(e: std::io::Error) -> PersistentAgentError {
    PersistentAgentError::Io(e.to_string())
}