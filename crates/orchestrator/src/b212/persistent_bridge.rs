//! Pont agents persistants ↔ workflow B212.

use crate::manager::AgentManager;
use crate::persistent::{PersistentAgent, PersistentAgentError};

use super::agents::B212_AGENTS;

/// Assure que les 6 agents domaine B212 existent (création idempotente).
pub async fn ensure_b212_agents(
    manager: &AgentManager,
) -> Result<Vec<PersistentAgent>, PersistentAgentError> {
    let mut created_or_existing = Vec::with_capacity(B212_AGENTS.len());
    for def in B212_AGENTS {
        let agent = match manager.get(def.id).await {
            Ok(existing) => existing,
            Err(PersistentAgentError::NotFound(_)) => {
                manager
                    .create_agent(def.id, def.name, def.role, None)
                    .await?
            }
            Err(e) => return Err(e),
        };
        created_or_existing.push(agent);
    }
    Ok(created_or_existing)
}

/// Réveille les agents B212 pour un cycle workflow (ordre canonique).
pub async fn wake_b212_agents_for_workflow(
    manager: &AgentManager,
) -> Result<Vec<PersistentAgent>, PersistentAgentError> {
    ensure_b212_agents(manager).await?;
    let mut awake = Vec::with_capacity(B212_AGENTS.len());
    for def in B212_AGENTS {
        let agent = manager.wake(def.id).await?;
        awake.push(agent);
    }
    Ok(awake)
}

/// Propage les résumés d'étapes via messagerie inter-agents (audit trail).
pub async fn relay_workflow_steps(
    manager: &AgentManager,
    steps: &[super::workflow::B212AgentStepReport],
) -> Result<(), PersistentAgentError> {
    if steps.len() < 2 {
        return Ok(());
    }
    for pair in steps.windows(2) {
        let from = &pair[0].agent_id;
        let to = &pair[1].agent_id;
        let body = format!(
            "[B212 handoff] {} → {}: {}",
            pair[0].agent_name, pair[1].agent_name, pair[0].summary
        );
        let _ = manager.send_message(from, to, &body).await?;
    }
    Ok(())
}