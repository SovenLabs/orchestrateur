use async_trait::async_trait;

use crate::error::SkillError;

/// Contexte minimal passé aux Skills (étendu en phases ultérieures).
#[derive(Debug, Clone, Default)]
pub struct SkillContext;

/// Résultat d'exécution d'une Skill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillOutput {
    /// Message ou payload textuel de retour.
    pub message: String,
}

/// Contrat d'une capacité extensible de l'orchestrateur.
///
/// `name` et `description` sont synchrones (métadonnées statiques).
/// `execute` est asynchrone (préparation aux appels réseau / IA futurs).
#[async_trait]
pub trait Skill: Send + Sync {
    /// Identifiant unique de la skill.
    fn name(&self) -> &'static str;

    /// Description lisible pour l'utilisateur ou l'UI.
    fn description(&self) -> &'static str;

    /// Exécute la skill.
    async fn execute(&self, ctx: &SkillContext) -> Result<SkillOutput, SkillError>;
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

#[async_trait]
impl Skill for NoopSkill {
    fn name(&self) -> &'static str {
        "noop"
    }

    fn description(&self) -> &'static str {
        "Skill de démonstration sans effet."
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
        let out = skill.execute(&SkillContext).await.unwrap();
        assert_eq!(out.message, "noop ok");
    }
}
