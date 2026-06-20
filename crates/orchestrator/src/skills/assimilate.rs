use async_trait::async_trait;

use crate::deps::AppDependencies;
use crate::error::SkillError;
use crate::skills::skill::{map_orchestrator_error, Skill, SkillContext, SkillOutput};
use crate::use_cases::AssimilateFromText;

/// Skill : assimile du texte brut via le provider LLM configuré.
pub struct AssimilateSkill {
    deps: AppDependencies,
}

impl AssimilateSkill {
    /// Crée la skill avec les dépendances injectées.
    #[must_use]
    pub fn new(deps: AppDependencies) -> Self {
        Self { deps }
    }
}

#[async_trait]
impl Skill for AssimilateSkill {
    fn name(&self) -> &'static str {
        "assimilate"
    }

    fn description(&self) -> &'static str {
        "Assimile du texte brut (paramètre text requis, tags optionnels)."
    }

    async fn execute(&self, ctx: &SkillContext) -> Result<SkillOutput, SkillError> {
        let text = ctx
            .text
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .ok_or_else(|| SkillError::ExecutionFailed("paramètre text requis".into()))?;

        let prompt = if ctx.tags.is_empty() {
            text.to_string()
        } else {
            format!(
                "{text}\n\nTags suggérés : {}",
                ctx.tags.join(", ")
            )
        };

        let (memory, _events) = AssimilateFromText::new(self.deps.clone())
            .execute(&prompt, None)
            .await
            .map_err(|err| map_orchestrator_error(&err))?;

        Ok(SkillOutput {
            message: format!("Assimilé : {} ({})", memory.title, memory.id),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockBundle;

    #[tokio::test]
    async fn assimilate_skill_requires_text() {
        let skill = AssimilateSkill::new(MockBundle::new().into_deps());
        let err = skill
            .execute(&SkillContext::default())
            .await
            .unwrap_err();
        assert_eq!(
            err,
            SkillError::ExecutionFailed("paramètre text requis".into())
        );
    }

    #[tokio::test]
    async fn assimilate_skill_creates_memory() {
        let skill = AssimilateSkill::new(MockBundle::new().into_deps());
        let out = skill
            .execute(&SkillContext {
                text: Some("Nouveau souvenir via skill.".into()),
                tags: vec!["test".into()],
                ..SkillContext::default()
            })
            .await
            .unwrap();
        assert!(out.message.contains("Assimilé"));
        assert!(out.message.contains("Nouveau souvenir"));
    }
}