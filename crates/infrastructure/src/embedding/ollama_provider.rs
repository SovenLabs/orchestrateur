use std::time::Duration;

use async_trait::async_trait;
use cortex::{Embedding, EmbeddingCapabilities, EmbeddingError, EmbeddingProvider};
use reqwest::Client;
use serde::Deserialize;
use tracing::instrument;

use crate::http_retry::with_retry;

/// Provider d'embeddings via l'API Ollama (`/api/embeddings`).
pub struct OllamaEmbeddingProvider {
    client: Client,
    base_url: String,
    model: String,
    timeout: Duration,
    max_retries: u32,
    dimension: Option<usize>,
}

impl OllamaEmbeddingProvider {
    /// Crée un provider Ollama avec client HTTP partagé.
    #[must_use]
    pub fn new(
        client: Client,
        base_url: impl Into<String>,
        model: impl Into<String>,
        timeout_secs: u64,
        max_retries: u32,
        dimension: Option<usize>,
    ) -> Self {
        Self {
            client,
            base_url: base_url.into(),
            model: model.into(),
            timeout: Duration::from_secs(timeout_secs),
            max_retries,
            dimension,
        }
    }

    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    async fn request_embedding(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let body = serde_json::json!({
            "model": self.model,
            "prompt": text,
        });
        let url = self.url("/api/embeddings");
        let client = self.client.clone();
        let timeout = self.timeout;
        let max_retries = self.max_retries;

        let response = tokio::time::timeout(timeout, async {
            with_retry("ollama", max_retries, || {
                let client = client.clone();
                let body = body.clone();
                let url = url.clone();
                async move { client.post(&url).json(&body).send().await }
            })
            .await
        })
        .await
        .map_err(|_| EmbeddingError::Network {
            provider: "ollama".into(),
            message: "timeout".into(),
            source: None,
        })?
        .map_err(|e| EmbeddingError::Network {
            provider: "ollama".into(),
            message: e.to_string(),
            source: Some(Box::new(e)),
        })?;

        if !response.status().is_success() {
            return Err(EmbeddingError::Unavailable {
                provider: "ollama".into(),
                message: format!("HTTP {}", response.status()),
            });
        }

        let parsed: OllamaEmbeddingResponse = response.json().await.map_err(|e| {
            EmbeddingError::InvalidResponse {
                provider: "ollama".into(),
                message: e.to_string(),
            }
        })?;

        Ok(parsed.embedding)
    }
}

#[derive(Debug, Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

#[async_trait]
impl EmbeddingProvider for OllamaEmbeddingProvider {
    fn name(&self) -> &'static str {
        "ollama"
    }

    fn capabilities(&self) -> EmbeddingCapabilities {
        EmbeddingCapabilities {
            supports_batch: false,
            supports_instruction: true,
            typical_dimensions: self.dimension,
            ..Default::default()
        }
    }

    #[instrument(skip(self, text))]
    async fn embed(&self, text: &str) -> Result<Embedding, EmbeddingError> {
        let vector = self.request_embedding(text).await?;
        Ok(Embedding::new(vector))
    }

    async fn embed_with_instruction(
        &self,
        text: &str,
        instruction: &str,
    ) -> Result<Embedding, EmbeddingError> {
        let combined = format!("{instruction}\n{text}");
        self.embed(&combined).await
    }
}