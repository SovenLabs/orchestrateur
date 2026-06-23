//! Worker tick — délégations, cron, background agents (Phase 2b).

mod cron;
mod delegation;

pub use cron::{run_due_cron_jobs, CronTickReport};
pub use delegation::{drain_delegations, DelegationDrainReport};

use std::sync::Arc;

use crate::facade::OrchestratorFacade;
use crate::persistent::{AgentIdentity, AgentStatus, PersistentAgentError};

/// Rapport agrégé d'un cycle worker.
#[derive(Debug, Clone, Default)]
pub struct AgentTickReport {
    /// Agents passés en background.
    pub agents_background: usize,
    /// Inbox traitées (auto-turn).
    pub inbox_turns: usize,
    /// Délégations complétées.
    pub delegations_completed: usize,
    /// Jobs cron exécutés.
    pub cron_ran: usize,
}

/// Exécute un cycle complet du worker agents.
pub async fn run_agent_tick(facade: &OrchestratorFacade) -> Result<AgentTickReport, PersistentAgentError> {
    let config = &facade.deps().config.agents;
    if !config.enabled {
        return Ok(AgentTickReport::default());
    }

    let manager = facade.agent_manager().await?;
    let mut report = AgentTickReport::default();

    for agent in manager.list().await? {
        if !matches!(agent.status(), AgentStatus::Awake | AgentStatus::Background) {
            continue;
        }
        let _ = manager.background(agent.id()).await?;
        report.agents_background += 1;

        if config.auto_turn_on_inbox {
            let messages = manager.receive_messages(agent.id(), false).await?;
            for msg in messages.iter().filter(|m| !m.read) {
                let prompt = format!("Message de {} : {}", msg.from, msg.body);
                if facade.agent_turn_for(agent.id(), &prompt).await.is_ok() {
                    report.inbox_turns += 1;
                }
            }
            let _ = manager.receive_messages(agent.id(), true).await?;
        }
    }

    let workspace = &facade.deps().config.workspace_root;
    let default_id = &config.default_worker_id;
    let deleg = drain_delegations(facade, workspace, default_id).await?;
    report.delegations_completed = deleg.completed;

    let cron = run_due_cron_jobs(facade, workspace, default_id).await?;
    report.cron_ran = cron.ran;

    Ok(report)
}

/// Démarre la boucle tick en arrière-plan (daemon).
pub fn spawn_agent_tick_if_enabled(facade: Arc<OrchestratorFacade>) {
    let config = facade.deps().config.agents.clone();
    if !config.enabled {
        return;
    }
    let interval_secs = config.tick_interval_secs.max(10);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
        loop {
            interval.tick().await;
            match run_agent_tick(&facade).await {
                Ok(report) => tracing::debug!(?report, "agent tick terminé"),
                Err(err) => tracing::warn!(%err, "agent tick échoué"),
            }
        }
    });
}