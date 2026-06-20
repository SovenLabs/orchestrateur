use crate::deps::AppDependencies;
use crate::error::OrchestratorError;
use crate::use_cases::assimilate_from_draft::{AssimilateFromDraft, AssimilationResult};

/// Prompt système par défaut pour la génération de [`MemoryDraft`] structuré.
pub const DEFAULT_ASSIMILATION_SYSTEM_PROMPT: &str = r#"Tu es l'assistant d'assimilation de l'Orchestrateur.
Produis UNIQUEMENT un objet JSON valide avec les champs :
- title (string, non vide)
- content (string markdown, non vide)
- tags (array de strings, minuscules sans espaces)
- backlinks (array optionnel d'objets { target: uuid-string, score: 0.0-1.0, kind: "semantic"|"explicit_wikilink" })
Ne produis aucun texte hors JSON."#;

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
        user_prompt: &str,
        system_prompt: Option<&str>,
    ) -> Result<AssimilationResult, OrchestratorError> {
        let system = system_prompt.unwrap_or(DEFAULT_ASSIMILATION_SYSTEM_PROMPT);
        let provider = self.deps.llm.name();
        tracing::info!(provider, "assimilation LLM démarrée");

        let draft = self
            .deps
            .llm
            .generate_memory_draft(system, user_prompt)
            .await?;

        if let Some(usage) = self.deps.llm.last_usage() {
            self.deps.events.publish_llm_usage(&usage);
        }

        tracing::info!(provider, title = %draft.title, "MemoryDraft généré");
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
            .execute("Architecture hexagonale durable", None)
            .await
            .unwrap();
        assert!(!memory.title.is_empty());
        assert!(memory.content.contains("hexagonale"));
        assert!(events.iter().any(|e| matches!(e, DomainEvent::MemoryAssimilated(_))));
    }
}