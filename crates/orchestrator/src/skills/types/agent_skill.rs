use async_trait::async_trait;

use crate::error::SkillError;
use crate::skills::context::SkillHostContext;
use crate::skills::metadata::SkillType;
use crate::skills::r#trait::{Skill, TypedSkill};
use crate::skills::skill::{SkillContext, SkillOutput};

/// Skill injectée dans un agent persistant.
#[async_trait]
pub trait AgentSkill: Skill {
    /// Identifiants agents autorisés (vide = tous).
    fn target_agent_ids(&self) -> &[String];

    /// Hook post-chargement pour l'agent cible.
    async fn on_agent_load(&self, agent_id: &str) -> Result<(), SkillError> {
        let _ = agent_id;
        Ok(())
    }
}

/// Adaptateur agent avec filtrage par `agent_id`.
pub struct AgentSkillAdapter<S> {
    inner: S,
}

impl<S> AgentSkillAdapter<S> {
    /// Enveloppe une skill agent.
    #[must_use]
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<S: AgentSkill + Send + Sync> Skill for AgentSkillAdapter<S> {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn metadata(&self) -> crate::skills::metadata::SkillMetadata {
        let mut meta = self.inner.metadata();
        meta.skill_type = SkillType::Agent;
        meta.agent_ids = self.inner.target_agent_ids().to_vec();
        meta
    }

    async fn execute(&self, ctx: &SkillContext) -> Result<SkillOutput, SkillError> {
        self.inner.execute(ctx).await
    }
}

#[async_trait]
impl<S: AgentSkill + Send + Sync> TypedSkill for AgentSkillAdapter<S> {
    fn typed_kind(&self) -> SkillType {
        SkillType::Agent
    }

    async fn on_load(&self, host: &SkillHostContext) -> Result<(), SkillError> {
        if let Some(agent_id) = host.agent_id.as_deref() {
            let allowed = self.inner.target_agent_ids();
            if allowed.is_empty() || allowed.iter().any(|id| id == agent_id) {
                return self.inner.on_agent_load(agent_id).await;
            }
        }
        Ok(())
    }
}