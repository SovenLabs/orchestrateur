use std::sync::Arc;

use async_trait::async_trait;
use cortex::{Embedding, EmbeddingCapabilities, EmbeddingError, EmbeddingProvider};
use tracing::warn;

/// Chaîne de providers avec fallback ordonné (primary → fallbacks).
pub struct ChainedEmbeddingProvider {
    providers: Vec<Arc<dyn EmbeddingProvider>>,
}

impl ChainedEmbeddingProvider {
    /// Crée une chaîne à partir d'au moins un provider.
    ///
    /// # Panics
    ///
    /// Panique si la liste est vide (construction invalide).
    #[must_use]
    pub fn new(providers: Vec<Arc<dyn EmbeddingProvider>>) -> Self {
        assert!(!providers.is_empty(), "au moins un embedding provider requis");
        Self { providers }
    }
}

#[async_trait]
impl EmbeddingProvider for ChainedEmbeddingProvider {
    fn name(&self) -> &'static str {
        "chain"
    }

    fn capabilities(&self) -> EmbeddingCapabilities {
        self.providers
            .first()
            .map(|p| p.capabilities())
            .unwrap_or_default()
    }

    async fn embed(&self, text: &str) -> Result<Embedding, EmbeddingError> {
        let mut last_err = None;
        for provider in &self.providers {
            match provider.embed(text).await {
                Ok(e) => return Ok(e),
                Err(e) => {
                    warn!(provider = provider.name(), error = %e, "embedding fallback");
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.unwrap_or_else(|| EmbeddingError::Internal {
            provider: "chain".into(),
            message: "aucun provider".into(),
        }))
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbeddingError> {
        let mut out = Vec::with_capacity(texts.len());
        for text in texts {
            out.push(self.embed(text).await?);
        }
        Ok(out)
    }
}