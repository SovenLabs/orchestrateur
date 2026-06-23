//! Tours LLM pour agents persistants (personality + session dédiée).

use std::sync::Arc;

use crate::agent::{
    AgentConfig, AgentLoop, AgentStreamSink, AgentTurnRequest, AgentTurnResult,
};
use crate::persistent::{AgentIdentity, AgentStatus};

use super::{AgentManager, PersistentAgentError};

impl AgentManager {
    /// Exécute un tour LLM pour un agent persistant.
    ///
    /// Réveille automatiquement l'agent s'il est en veille.
    pub async fn run_turn(
        &self,
        id: &str,
        message: &str,
        skills: Option<Arc<crate::skills::SkillRegistry>>,
    ) -> Result<AgentTurnResult, PersistentAgentError> {
        self.run_turn_with_stream(id, message, skills, AgentStreamSink::noop())
            .await
    }

    /// Exécute un tour avec streaming d'événements (gateway Phase 2b).
    pub async fn run_turn_with_stream(
        &self,
        id: &str,
        message: &str,
        skills: Option<Arc<crate::skills::SkillRegistry>>,
        stream: AgentStreamSink,
    ) -> Result<AgentTurnResult, PersistentAgentError> {
        let _agent = self.prepare_for_turn(id).await?;
        let personality = load_personality(&_agent).await?;
        let session_key = _agent.session_key()?;
        let config = AgentConfig::from_settings(&self.deps.config.agent);

        let loop_agent = AgentLoop::new(self.deps.clone(), config, skills);
        loop_agent
            .run_turn_with_stream(
                AgentTurnRequest {
                    session_key,
                    message: message.to_string(),
                    personality_prefix: Some(personality),
                },
                stream,
            )
            .await
            .map_err(PersistentAgentError::from)
    }

    async fn prepare_for_turn(&self, id: &str) -> Result<crate::persistent::PersistentAgent, PersistentAgentError> {
        let status = self.get(id).await?.status();
        if status == AgentStatus::Sleeping {
            self.wake(id).await?;
        }
        self.get(id).await
    }
}

async fn load_personality(agent: &crate::persistent::PersistentAgent) -> Result<String, PersistentAgentError> {
    let path = agent.root.join("personality.md");
    if !path.exists() {
        return Ok(String::new());
    }
    tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| PersistentAgentError::Io(e.to_string()))
}

impl From<crate::agent::AgentError> for PersistentAgentError {
    fn from(err: crate::agent::AgentError) -> Self {
        match err {
            crate::agent::AgentError::PersistentAgent(e) => e,
            crate::agent::AgentError::Cortex(e) => Self::Cortex(e),
            crate::agent::AgentError::Orchestrator(e) => Self::Orchestrator(e),
            other => Self::Io(other.to_string()),
        }
    }
}