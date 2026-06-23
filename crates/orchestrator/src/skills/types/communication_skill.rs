use async_trait::async_trait;

use crate::error::SkillError;
use crate::skills::context::SkillHostContext;
use crate::skills::metadata::SkillType;
use crate::skills::r#trait::{Skill, TypedSkill};
use crate::skills::skill::{SkillContext, SkillOutput};

/// Skill améliorant la messagerie inter-agents.
#[async_trait]
pub trait CommunicationSkill: Skill {
    /// Priorité de traitement des messages (plus haut = plus tôt).
    fn message_priority(&self) -> u8 {
        0
    }

    /// Hook post-chargement messagerie.
    async fn on_messaging_register(&self) -> Result<(), SkillError> {
        Ok(())
    }
}

/// Adaptateur Communication.
pub struct CommunicationSkillAdapter<S> {
    inner: S,
}

impl<S> CommunicationSkillAdapter<S> {
    /// Enveloppe une skill communication.
    #[must_use]
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<S: CommunicationSkill + Send + Sync> Skill for CommunicationSkillAdapter<S> {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn metadata(&self) -> crate::skills::metadata::SkillMetadata {
        let mut meta = self.inner.metadata();
        meta.skill_type = SkillType::Communication;
        meta
    }

    async fn execute(&self, ctx: &SkillContext) -> Result<SkillOutput, SkillError> {
        self.inner.execute(ctx).await
    }
}

#[async_trait]
impl<S: CommunicationSkill + Send + Sync> TypedSkill for CommunicationSkillAdapter<S> {
    fn typed_kind(&self) -> SkillType {
        SkillType::Communication
    }

    async fn on_load(&self, _host: &SkillHostContext) -> Result<(), SkillError> {
        self.inner.on_messaging_register().await
    }
}