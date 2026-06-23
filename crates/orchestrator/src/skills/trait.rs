use async_trait::async_trait;

use crate::error::SkillError;

use super::metadata::{SkillMetadata, SkillType};
use super::skill::{SkillContext, SkillOutput, SkillSource};

/// Contrat d'une capacité extensible de l'orchestrateur (Phase 6).
#[async_trait]
pub trait Skill: Send + Sync {
    /// Identifiant unique de la skill.
    fn name(&self) -> &str;

    /// Description lisible pour l'utilisateur ou l'UI.
    fn description(&self) -> &str;

    /// Origine de la skill (builtin par défaut).
    fn source(&self) -> SkillSource {
        SkillSource::Builtin
    }

    /// Version optionnelle (plugins hub).
    fn version(&self) -> Option<&str> {
        None
    }

    /// Métadonnées enrichies.
    fn metadata(&self) -> SkillMetadata {
        SkillMetadata::minimal(self.name(), self.description())
    }

    /// Type fonctionnel (dérivé des métadonnées par défaut).
    fn skill_type(&self) -> SkillType {
        self.metadata().skill_type
    }

    /// Exécute la skill.
    async fn execute(&self, ctx: &SkillContext) -> Result<SkillOutput, SkillError>;
}

/// Skill typée avec hook d'initialisation optionnel.
#[async_trait]
pub trait TypedSkill: Skill {
    /// Type fonctionnel explicite.
    fn typed_kind(&self) -> SkillType;

    /// Appelé après chargement dans le registre (extensions, injection agent…).
    async fn on_load(&self, _host: &super::context::SkillHostContext) -> Result<(), SkillError> {
        Ok(())
    }
}