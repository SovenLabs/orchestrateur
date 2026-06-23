use crate::error::{OrchestratorError, SkillError};

use crate::bridge::BridgeSkillContext;

use super::metadata::SkillMetadata;
use super::r#trait::Skill;

/// Convertit une erreur orchestrateur en erreur skill.
#[must_use]
pub(crate) fn map_orchestrator_error(err: &OrchestratorError) -> SkillError {
    SkillError::ExecutionFailed(err.to_string())
}

/// Paramètres optionnels passés aux skills opérationnelles.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SkillContext {
    /// Requête de recherche sémantique (`search`).
    pub query: Option<String>,
    /// Texte à assimiler (`assimilate`).
    pub text: Option<String>,
    /// Tags suggérés (filtre `search` ou contexte `assimilate`).
    pub tags: Vec<String>,
    /// Limite de résultats (`search`).
    pub limit: Option<usize>,
}

impl From<BridgeSkillContext> for SkillContext {
    fn from(ctx: BridgeSkillContext) -> Self {
        Self {
            query: ctx.query,
            text: ctx.text,
            tags: ctx.tags,
            limit: ctx.limit,
        }
    }
}

/// Résultat d'exécution d'une Skill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillOutput {
    /// Message ou payload textuel de retour.
    pub message: String,
}

/// Origine d'une skill dans le registre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillSource {
    /// Skill compilée dans l'orchestrateur.
    Builtin,
    /// Plugin chargé depuis le hub (subprocess).
    Hub,
    /// Plugin bibliothèque native (Phase 12).
    Native,
}

/// Métadonnées exposées par le registre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillEntry {
    /// Identifiant stable.
    pub name: String,
    /// Description lisible.
    pub description: String,
    /// Origine (`builtin` / `hub`).
    pub source: SkillSource,
    /// Version optionnelle (plugins hub).
    pub version: Option<String>,
    /// Type fonctionnel (Phase 6).
    pub skill_type: super::metadata::SkillType,
    /// Auteur optionnel.
    pub author: Option<String>,
    /// Dépendances déclarées.
    pub dependencies: Vec<String>,
}

impl SkillEntry {
    /// Construit depuis les métadonnées complètes d'une skill.
    #[must_use]
    pub fn from_metadata(meta: &SkillMetadata) -> Self {
        Self {
            name: meta.id.clone(),
            description: meta.description.clone(),
            source: meta.source,
            version: Some(meta.version.clone()),
            skill_type: meta.skill_type,
            author: meta.author.clone(),
            dependencies: meta.dependencies.clone(),
        }
    }
}

/// Skill de démonstration sans effet de bord.
pub struct NoopSkill;

impl NoopSkill {
    /// Crée la skill noop.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoopSkill {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Skill for NoopSkill {
    fn name(&self) -> &'static str {
        "noop"
    }

    fn description(&self) -> &'static str {
        "Skill de démonstration sans effet."
    }

    fn metadata(&self) -> SkillMetadata {
        SkillMetadata::minimal("noop", "Skill de démonstration sans effet.")
    }

    async fn execute(&self, _ctx: &SkillContext) -> Result<SkillOutput, SkillError> {
        Ok(SkillOutput {
            message: "noop ok".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn noop_skill_executes() {
        let skill = NoopSkill::new();
        assert_eq!(skill.name(), "noop");
        let out = skill.execute(&SkillContext::default()).await.unwrap();
        assert_eq!(out.message, "noop ok");
    }
}