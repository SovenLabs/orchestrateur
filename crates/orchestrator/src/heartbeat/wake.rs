//! Réveil d'un agent persistant.

use crate::deps::AppDependencies;
use crate::persistent::{AgentStatus, CortexAgentBridge, PersistentAgent, PersistentAgentError};

use super::{persist_lifecycle, AgentHeartbeat};

/// Réveille un agent : statut `Awake`, session Cortex prête, heartbeat mis à jour.
///
/// # Errors
///
/// Propage [`PersistentAgentError`] si la persistance ou la session échoue.
pub async fn wake_agent(
    deps: &AppDependencies,
    agent: &mut PersistentAgent,
) -> Result<(), PersistentAgentError> {
    CortexAgentBridge::ensure_ready(deps, agent).await?;
    let mut heartbeat = AgentHeartbeat::load(&agent.root).await?;
    heartbeat.mark_wake();
    persist_lifecycle(&agent.root, &mut agent.config, AgentStatus::Awake, &mut heartbeat).await
}