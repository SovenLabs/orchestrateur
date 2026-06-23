use crate::deps::AppDependencies;
use crate::error::OrchestratorError;
use crate::memory::INSIGHT_ASSIMILATION_SYSTEM_PROMPT;
use crate::use_cases::assimilate_from_draft::{AssimilateFromDraft, AssimilationResult};
use crate::use_cases::GenerateInsightDraft;

/// Prompt système legacy (rétrocompat tests).
pub const DEFAULT_ASSIMILATION_SYSTEM_PROMPT: &str = INSIGHT_ASSIMILATION_SYSTEM_PROMPT;

/// Use case : assimile du contenu brut via LLM puis pipeline Cortex complet.
pub struct AssimilateFromText {
    deps: AppDependencies,
}

impl AssimilateFromText {
    /// Crée le use case avec les dépendances injectées.
    #[must_use]
    pub fn new(deps: AppDependencies) -> Self {
        Self { deps }
    }

    /// Génère un [`MemoryDraft`] via le provider LLM configuré, puis assimile.
    ///
    /// # Errors
    ///
    /// Propage [`OrchestratorError`] si le LLM, la validation ou la persistance échoue.
    pub async fn execute(
        &self,
        text: &str,
        tags: &[String],
        system_prompt: Option<&str>,
    ) -> Result<AssimilationResult, OrchestratorError> {
        let provider = self.deps.llm.name();
        tracing::info!(provider, "assimilation LLM démarrée");

        let draft = GenerateInsightDraft::new(self.deps.clone())
            .execute(text, tags, None, system_prompt)
            .await?;

        AssimilateFromDraft::new(self.deps.clone())
            .execute(draft)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockBundle;
    use cortex::DomainEvent;

    #[tokio::test]
    async fn assimilates_text_via_mock_llm() {
        let bundle = MockBundle::new();
        let (memory, events) = AssimilateFromText::new(bundle.into_deps())
            .execute("Architecture hexagonale durable", &[], None)
            .await
            .unwrap();
        assert!(!memory.title.is_empty());
        assert!(memory.content.contains("hexagonale"));
        assert!(events
            .iter()
            .any(|e| matches!(e, DomainEvent::MemoryAssimilated(_))));
    }
}