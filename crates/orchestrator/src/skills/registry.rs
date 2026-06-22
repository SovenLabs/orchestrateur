use std::collections::HashMap;
use std::sync::Arc;

use tracing::warn;

use crate::config::OrchestratorConfig;
use crate::deps::AppDependencies;
use crate::error::SkillError;
use crate::skills::assimilate::AssimilateSkill;
use crate::skills::hub::SkillsHub;
use crate::skills::list_memories::ListMemoriesSkill;
use crate::skills::search::SearchMemoriesSkill;
use crate::skills::skill::{Skill, SkillContext, SkillEntry, SkillOutput, SkillSource};

/// Registre centralisé des Skills disponibles.
pub struct SkillRegistry {
    skills: HashMap<String, Arc<dyn Skill>>,
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

    /// Crée un registre avec les skills opérationnelles (`list_memories`, `search`, `assimilate`).
    #[must_use]
    pub fn with_operational_skills(deps: AppDependencies) -> Self {
        let mut registry = Self::with_defaults();
        registry.register(Arc::new(ListMemoriesSkill::new(deps.clone())));
        registry.register(Arc::new(SearchMemoriesSkill::new(deps.clone())));
        registry.register(Arc::new(AssimilateSkill::new(deps)));
        registry
    }

    /// Crée un registre opérationnel + plugins hub si activés.
    #[must_use]
    pub fn with_operational_skills_and_hub(deps: AppDependencies) -> Self {
        let config = deps.config.clone();
        let mut registry = Self::with_operational_skills(deps);
        if config.skills_hub.enabled && config.skills_hub.auto_load {
            match SkillsHub::load_into(&mut registry, &config) {
                Ok(count) => {
                    if count > 0 {
                        tracing::info!(count, "skills hub chargées");
                    }
                }
                Err(err) => warn!(%err, "échec chargement skills hub"),
            }
        }
        registry
    }

    /// Enregistre une skill (écrase si le nom existe déjà).
    pub fn register(&mut self, skill: Arc<dyn Skill>) {
        self.skills.insert(skill.name().to_string(), skill);
    }

    /// Liste les métadonnées des skills enregistrées.
    #[must_use]
    pub fn list(&self) -> Vec<SkillEntry> {
        let mut entries: Vec<_> = self
            .skills
            .values()
            .map(|skill| SkillEntry {
                name: skill.name().to_string(),
                description: skill.description().to_string(),
                source: skill.source(),
                version: skill.version().map(str::to_string),
            })
            .collect();
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        entries
    }

    /// Recharge les plugins hub (conserve les skills builtin).
    ///
    /// # Errors
    ///
    /// Propage les erreurs de scan hub.
    pub fn reload_hub(&mut self, config: &OrchestratorConfig) -> Result<usize, crate::skills::hub::HubError> {
        self.skills
            .retain(|_, skill| skill.source() == SkillSource::Builtin);
        SkillsHub::load_into(self, config)
    }

    /// Exécute une skill par son nom.
    ///
    /// # Errors
    ///
    /// Retourne [`SkillError::NotFound`] ou [`SkillError::ExecutionFailed`].
    pub async fn execute(&self, name: &str, ctx: &SkillContext) -> Result<SkillOutput, SkillError> {
        let skill = self
            .skills
            .get(name)
            .ok_or_else(|| SkillError::NotFound(name.to_string()))?;
        skill.execute(ctx).await
    }
}

impl Clone for SkillRegistry {
    fn clone(&self) -> Self {
        Self {
            skills: self.skills.clone(),
        }
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
    use crate::config::SkillsHubEntryConfig;
    use crate::skills::plugin::SubprocessPluginSkill;
    use crate::testing::MockBundle;

    #[tokio::test]
    async fn default_registry_has_noop() {
        let registry = SkillRegistry::with_defaults();
        let list = registry.list();
        assert!(list.iter().any(|entry| entry.name == "noop"));
        let out = registry
            .execute("noop", &SkillContext::default())
            .await
            .unwrap();
        assert_eq!(out.message, "noop ok");
    }

    #[tokio::test]
    async fn unknown_skill_returns_not_found() {
        let registry = SkillRegistry::new();
        let err = registry
            .execute("missing", &SkillContext::default())
            .await
            .unwrap_err();
        assert_eq!(err, SkillError::NotFound("missing".into()));
    }

    #[tokio::test]
    async fn hub_plugin_overrides_by_name() {
        let mut registry = SkillRegistry::with_defaults();
        let entry = SkillsHubEntryConfig {
            id: "noop".into(),
            description: "noop hub".into(),
            enabled: true,
            command: "echo".into(),
            args: vec!["hub-noop".into()],
            stdin_json: false,
            timeout_secs: 5,
        };
        registry.register(Arc::new(SubprocessPluginSkill::from_entry(entry)));
        let list = registry.list();
        let noop = list.iter().find(|e| e.name == "noop").unwrap();
        assert_eq!(noop.source, SkillSource::Hub);
    }

    #[test]
    fn operational_registry_lists_builtin_skills() {
        let registry = SkillRegistry::with_operational_skills(MockBundle::new().into_deps());
        let names: Vec<_> = registry.list().into_iter().map(|e| e.name).collect();
        for expected in ["noop", "list_memories", "search", "assimilate"] {
            assert!(names.iter().any(|n| n == expected));
        }
    }
}