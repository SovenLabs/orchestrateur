//! Mise en veille d'un agent persistant.

use crate::persistent::{AgentStatus, PersistentAgent, PersistentAgentError};

use super::{persist_lifecycle, AgentHeartbeat};

/// Met un agent en veille : statut `Sleeping`, heartbeat mis à jour.
pub async fn sleep_agent(agent: &mut PersistentAgent) -> Result<(), PersistentAgentError> {
    let mut heartbeat = AgentHeartbeat::load(&agent.root).await?;
    heartbeat.mark_sleep();
    persist_lifecycle(&agent.root, &mut agent.config, AgentStatus::Sleeping, &mut heartbeat).await
}