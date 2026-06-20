use std::collections::HashMap;
use std::sync::Arc;

use crate::error::SkillError;
use crate::skills::skill::{Skill, SkillContext, SkillOutput};

/// Registre centralisé des Skills disponibles.
pub struct SkillRegistry {
    skills: HashMap<&'static str, Arc<dyn Skill>>,
}

impl SkillRegistry {
    /// Crée un registre vide.
    #[must_use]
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    /// Crée un registre avec la skill `noop` pré-enregistrée.
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Arc::new(super::skill::NoopSkill::new()));
        registry
    }

    /// Enregistre une skill (écrase si le nom existe déjà).
    pub fn register(&mut self, skill: Arc<dyn Skill>) {
        self.skills.insert(skill.name(), skill);
    }

    /// Liste les paires (nom, description) des skills enregistrées.
    #[must_use]
    pub fn list(&self) -> Vec<(&'static str, &'static str)> {
        let mut entries: Vec<_> = self
            .skills
            .values()
            .map(|s| (s.name(), s.description()))
            .collect();
        entries.sort_by_key(|(name, _)| *name);
        entries
    }

    /// Exécute une skill par son nom.
    ///
    /// # Errors
    ///
    /// Retourne [`SkillError::NotFound`] ou [`SkillError::ExecutionFailed`].
    pub async fn execute(
        &self,
        name: &str,
        ctx: &SkillContext,
    ) -> Result<SkillOutput, SkillError> {
        let skill = self
            .skills
            .get(name)
            .ok_or_else(|| SkillError::NotFound(name.to_string()))?;
        skill.execute(ctx).await
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn default_registry_has_noop() {
        let registry = SkillRegistry::with_defaults();
        let list = registry.list();
        assert!(list.iter().any(|(n, _)| *n == "noop"));
        let out = registry.execute("noop", &SkillContext).await.unwrap();
        assert_eq!(out.message, "noop ok");
    }

    #[tokio::test]
    async fn unknown_skill_returns_not_found() {
        let registry = SkillRegistry::new();
        let err = registry.execute("missing", &SkillContext).await.unwrap_err();
        assert_eq!(err, SkillError::NotFound("missing".into()));
    }
}