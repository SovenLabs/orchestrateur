use std::sync::Arc;

use async_trait::async_trait;
use orchestrator::{
    ChatMessage, LlmCapabilities, LlmError, LlmProvider, LlmUsageRecorded, MemoryDraft,
};
use tracing::warn;

/// Chaîne LLM avec fallback ordonné.
pub struct ChainedLlmProvider {
    providers: Vec<Arc<dyn LlmProvider>>,
}

impl ChainedLlmProvider {
    /// Crée une chaîne à partir d'au moins un provider.
    ///
    /// # Panics
    ///
    /// Panique si la liste est vide.
    #[must_use]
    pub fn new(providers: Vec<Arc<dyn LlmProvider>>) -> Self {
        assert!(!providers.is_empty(), "au moins un LLM provider requis");
        Self { providers }
    }
}

#[async_trait]
impl LlmProvider for ChainedLlmProvider {
    fn name(&self) -> &'static str {
        "chain"
    }

    fn capabilities(&self) -> LlmCapabilities {
        self.providers
            .first()
            .map(|p| p.capabilities())
            .unwrap_or_default()
    }

    async fn generate_memory_draft(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<MemoryDraft, LlmError> {
        let mut last_err = None;
        for provider in &self.providers {
            match provider
                .generate_memory_draft(system_prompt, user_prompt)
                .await
            {
                Ok(d) => return Ok(d),
                Err(e) if e.should_fallback() => {
                    warn!(provider = provider.name(), error = %e, "LLM fallback");
                    last_err = Some(e);
                }
                Err(e) => return Err(e),
            }
        }
        Err(last_err.unwrap_or_else(|| LlmError::ProviderError {
            provider: "chain".into(),
            message: "aucun provider".into(),
        }))
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<String, LlmError> {
        let mut last_err = None;
        for provider in &self.providers {
            match provider.chat(messages).await {
                Ok(s) => return Ok(s),
                Err(e) if e.should_fallback() => {
                    warn!(provider = provider.name(), error = %e, "chat LLM fallback");
                    last_err = Some(e);
                }
                Err(e) => return Err(e),
            }
        }
        Err(last_err.unwrap_or_else(|| LlmError::ProviderError {
            provider: "chain".into(),
            message: "aucun provider".into(),
        }))
    }

    fn last_usage(&self) -> Option<LlmUsageRecorded> {
        self.providers.iter().find_map(|p| p.last_usage())
    }
}
