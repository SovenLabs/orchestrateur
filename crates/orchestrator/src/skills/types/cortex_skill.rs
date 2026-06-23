use async_trait::async_trait;

use crate::cortex_extensions::CortexExtensionRegistry;
use crate::error::SkillError;
use crate::skills::context::SkillHostContext;
use crate::skills::metadata::SkillType;
use crate::skills::r#trait::{Skill, TypedSkill};
use crate::skills::skill::{SkillContext, SkillOutput};

/// Skill qui étend le Cortex via [`CortexExtensionRegistry`].
#[async_trait]
pub trait CortexSkill: Skill {
    /// Enregistre les hooks Cortex.
    async fn register_extensions(
        &self,
        registry: &CortexExtensionRegistry,
    ) -> Result<(), SkillError>;
}

/// Adaptateur pour charger une skill Cortex dans le registre d'extensions.
pub struct CortexSkillAdapter<S> {
    inner: S,
}

impl<S> CortexSkillAdapter<S> {
    /// Enveloppe une skill Cortex.
    #[must_use]
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<S: CortexSkill + Send + Sync> Skill for CortexSkillAdapter<S> {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn metadata(&self) -> crate::skills::metadata::SkillMetadata {
        let mut meta = self.inner.metadata();
        meta.skill_type = SkillType::Cortex;
        meta
    }

    async fn execute(&self, ctx: &SkillContext) -> Result<SkillOutput, SkillError> {
        self.inner.execute(ctx).await
    }
}

#[async_trait]
impl<S: CortexSkill + Send + Sync> TypedSkill for CortexSkillAdapter<S> {
    fn typed_kind(&self) -> SkillType {
        SkillType::Cortex
    }

    async fn on_load(&self, host: &SkillHostContext) -> Result<(), SkillError> {
        self.inner
            .register_extensions(&host.cortex_extensions)
            .await
    }
}