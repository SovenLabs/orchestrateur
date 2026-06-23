//! Enregistrement d'un nouvel agent persistant.

use std::path::Path;

use crate::persistent::{
    AgentStructure, PersistentAgent, PersistentAgentConfig, PersistentAgentError,
};

/// Crée le dossier agent et retourne l'entité chargée.
///
/// # Errors
///
/// Propage [`PersistentAgentError`] si l'agent existe déjà ou si l'écriture échoue.
pub async fn register_agent(
    agents_dir: &Path,
    config: PersistentAgentConfig,
) -> Result<PersistentAgent, PersistentAgentError> {
    let root = AgentStructure::scaffold(agents_dir, &config).await?;
    Ok(PersistentAgent::from_config(root, config))
}