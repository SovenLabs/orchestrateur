use async_trait::async_trait;

use crate::error::SkillError;
use crate::skills::context::SkillHostContext;
use crate::skills::metadata::SkillType;
use crate::skills::r#trait::{Skill, TypedSkill};
use crate::skills::skill::{SkillContext, SkillOutput};

/// Skill branchée sur le protocole B212.
#[async_trait]
pub trait B212Skill: Skill {
    /// Modules B212 exposés par la skill.
    fn b212_modules(&self) -> &[String];

    /// Hook post-chargement B212.
    async fn on_b212_register(&self) -> Result<(), SkillError> {
        Ok(())
    }
}

/// Adaptateur B212.
pub struct B212SkillAdapter<S> {
    inner: S,
}

impl<S> B212SkillAdapter<S> {
    /// Enveloppe une skill B212.
    #[must_use]
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<S: B212Skill + Send + Sync> Skill for B212SkillAdapter<S> {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn metadata(&self) -> crate::skills::metadata::SkillMetadata {
        let mut meta = self.inner.metadata();
        meta.skill_type = SkillType::B212;
        meta
    }

    async fn execute(&self, ctx: &SkillContext) -> Result<SkillOutput, SkillError> {
        self.inner.execute(ctx).await
    }
}

#[async_trait]
impl<S: B212Skill + Send + Sync> TypedSkill for B212SkillAdapter<S> {
    fn typed_kind(&self) -> SkillType {
        SkillType::B212
    }

    async fn on_load(&self, _host: &SkillHostContext) -> Result<(), SkillError> {
        self.inner.on_b212_register().await
    }
}