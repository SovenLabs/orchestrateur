use async_trait::async_trait;

use crate::deps::AppDependencies;
use crate::error::SkillError;
use crate::skills::skill::{map_orchestrator_error, Skill, SkillContext, SkillOutput};
use crate::use_cases::ListMemories;

/// Skill : liste toutes les mémoires persistées.
pub struct ListMemoriesSkill {
    deps: AppDependencies,
}

impl ListMemoriesSkill {
    /// Crée la skill avec les dépendances injectées.
    #[must_use]
    pub fn new(deps: AppDependencies) -> Self {
        Self { deps }
    }
}

#[async_trait]
impl Skill for ListMemoriesSkill {
    fn name(&self) -> &'static str {
        "list_memories"
    }

    fn description(&self) -> &'static str {
        "Liste les mémoires persistées (titre et identifiant)."
    }

    async fn execute(&self, _ctx: &SkillContext) -> Result<SkillOutput, SkillError> {
        let memories = ListMemories::new(self.deps.clone())
            .execute()
            .await
            .map_err(|err| map_orchestrator_error(&err))?;

        if memories.is_empty() {
            return Ok(SkillOutput {
                message: "Aucune mémoire.".into(),
            });
        }

        let lines: Vec<String> = memories
            .iter()
            .map(|m| format!("{} — {}", m.id, m.title))
            .collect();
        Ok(SkillOutput {
            message: format!("{} mémoire(s):\n{}", memories.len(), lines.join("\n")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockBundle;
    use cortex::{Memory, MemoryRepository};

    #[tokio::test]
    async fn list_memories_skill_returns_titles() {
        let bundle = MockBundle::new();
        let mem = Memory::new("Alpha", "contenu").unwrap();
        bundle.memory_repo.save(&mem).await.unwrap();
        let skill = ListMemoriesSkill::new(bundle.into_deps());
        let out = skill.execute(&SkillContext::default()).await.unwrap();
        assert!(out.message.contains("Alpha"));
        assert!(out.message.contains("1 mémoire"));
    }
}