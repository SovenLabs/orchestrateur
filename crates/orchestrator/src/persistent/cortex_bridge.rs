//! Liaison entre un agent persistant et les ports Cortex (session + mémoires).

use crate::deps::AppDependencies;

use super::memory::{ensure_memories_dir, AgentMemoryStore};
use super::{PersistentAgent, PersistentAgentError};

/// Initialise les ressources Cortex d'un agent (session SQLite + dossier mémoires).
pub struct CortexAgentBridge;

impl CortexAgentBridge {
    /// Assure qu'une session agent existe et que le dossier mémoires est prêt.
    ///
    /// # Errors
    ///
    /// Propage [`PersistentAgentError`] si la session ou l'I/O échoue.
    pub async fn ensure_ready(
        deps: &AppDependencies,
        agent: &PersistentAgent,
    ) -> Result<(), PersistentAgentError> {
        let key = agent.session_key()?;
        deps.session_repo.get_or_create(&key).await?;
        ensure_memories_dir(&agent.root).await
    }

    /// Retourne un store mémoire scopingé pour l'agent.
    #[must_use]
    pub fn memory_store(agent: PersistentAgent, deps: AppDependencies) -> AgentMemoryStore {
        AgentMemoryStore::new(agent, deps)
    }
}