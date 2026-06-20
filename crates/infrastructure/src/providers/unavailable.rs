//! Providers « stub » — l'application démarre, les appels retournent `Unavailable`.

use async_trait::async_trait;
use cortex::{Embedding, EmbeddingCapabilities, EmbeddingError, EmbeddingProvider};
use orchestrator::{ChatMessage, LlmCapabilities, LlmError, LlmProvider, MemoryDraft};

/// LLM indisponible au démarrage (aucun provider résolu).
pub struct UnavailableLlmProvider {
    reason: String,
}

impl UnavailableLlmProvider {
    /// Crée un stub avec motif lisible (logs / health).
    #[must_use]
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

#[async_trait]
impl LlmProvider for UnavailableLlmProvider {
    fn name(&self) -> &'static str {
        "unavailable"
    }

    fn capabilities(&self) -> LlmCapabilities {
        LlmCapabilities::default()
    }

    async fn generate_memory_draft(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
    ) -> Result<MemoryDraft, LlmError> {
        Err(LlmError::Unavailable {
            provider: self.name().into(),
            message: self.reason.clone(),
        })
    }

    async fn chat(&self, _messages: &[ChatMessage]) -> Result<String, LlmError> {
        Err(LlmError::Unavailable {
            provider: self.name().into(),
            message: self.reason.clone(),
        })
    }
}

/// Embeddings indisponibles au démarrage.
pub struct UnavailableEmbeddingProvider {
    reason: String,
}

impl UnavailableEmbeddingProvider {
    /// Crée un stub avec motif lisible.
    #[must_use]
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

#[async_trait]
impl EmbeddingProvider for UnavailableEmbeddingProvider {
    fn name(&self) -> &'static str {
        "unavailable"
    }

    fn capabilities(&self) -> EmbeddingCapabilities {
        EmbeddingCapabilities::default()
    }

    async fn embed(&self, _text: &str) -> Result<Embedding, EmbeddingError> {
        Err(EmbeddingError::Unavailable {
            provider: self.name().into(),
            message: self.reason.clone(),
        })
    }
}
