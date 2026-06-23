use async_trait::async_trait;
use cortex::{SearchFilter, Tag};

use crate::deps::AppDependencies;
use crate::error::SkillError;
use crate::skills::metadata::{SkillMetadata, SkillType};
use crate::skills::r#trait::Skill;
use crate::skills::skill::{map_orchestrator_error, SkillContext, SkillOutput};
use crate::use_cases::SearchMemories;

/// Limite par défaut des résultats de recherche via skill.
const DEFAULT_SEARCH_LIMIT: usize = 10;

/// Skill : recherche sémantique hybride.
pub struct SearchMemoriesSkill {
    deps: AppDependencies,
}

impl SearchMemoriesSkill {
    /// Crée la skill avec les dépendances injectées.
    #[must_use]
    pub fn new(deps: AppDependencies) -> Self {
        Self { deps }
    }
}

#[async_trait]
impl Skill for SearchMemoriesSkill {
    fn name(&self) -> &'static str {
        "search"
    }

    fn description(&self) -> &'static str {
        "Recherche sémantique (paramètre query requis, limit et tags optionnels)."
    }

    fn metadata(&self) -> SkillMetadata {
        let mut meta = SkillMetadata::minimal(self.name(), self.description());
        meta.skill_type = SkillType::Cortex;
        meta
    }

    async fn execute(&self, ctx: &SkillContext) -> Result<SkillOutput, SkillError> {
        let query = ctx
            .query
            .as_deref()
            .map(str::trim)
            .filter(|q| !q.is_empty())
            .ok_or_else(|| SkillError::ExecutionFailed("paramètre query requis".into()))?;

        let limit = ctx.limit.unwrap_or(DEFAULT_SEARCH_LIMIT);
        let mut filter = SearchFilter {
            limit: Some(limit),
            ..Default::default()
        };
        for tag in &ctx.tags {
            let parsed = Tag::new(tag).map_err(|err| {
                SkillError::ExecutionFailed(format!("tag invalide `{tag}`: {err}"))
            })?;
            filter.tags.push(parsed);
        }

        let hits = SearchMemories::new(self.deps.clone())
            .execute(query, &filter)
            .await
            .map_err(|err| map_orchestrator_error(&err))?;

        if hits.is_empty() {
            return Ok(SkillOutput {
                message: format!("Aucun résultat pour « {query} »."),
            });
        }

        let lines: Vec<String> = hits
            .iter()
            .map(|hit| {
                let snippet = hit
                    .snippet
                    .as_deref()
                    .unwrap_or("—");
                format!("{:.3} {} — {snippet}", hit.score, hit.memory_id)
            })
            .collect();
        Ok(SkillOutput {
            message: format!(
                "{} résultat(s) pour « {query} »:\n{}",
                hits.len(),
                lines.join("\n")
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockBundle;
    use cortex::{Memory, MemoryRepository};

    #[tokio::test]
    async fn search_skill_requires_query() {
        let skill = SearchMemoriesSkill::new(MockBundle::new().into_deps());
        let err = skill
            .execute(&SkillContext::default())
            .await
            .unwrap_err();
        assert_eq!(
            err,
            SkillError::ExecutionFailed("paramètre query requis".into())
        );
    }

    #[tokio::test]
    async fn search_skill_finds_memory() {
        let bundle = MockBundle::new();
        let mem = Memory::new("Rust", "Contenu indexé pour test.").unwrap();
        bundle.memory_repo.save(&mem).await.unwrap();
        let skill = SearchMemoriesSkill::new(bundle.into_deps());
        let out = skill
            .execute(&SkillContext {
                query: Some("indexé".into()),
                ..SkillContext::default()
            })
            .await
            .unwrap();
        assert!(out.message.contains("résultat"));
    }
}