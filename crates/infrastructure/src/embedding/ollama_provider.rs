use std::time::Duration;

use async_trait::async_trait;
use cortex::{Embedding, EmbeddingCapabilities, EmbeddingError, EmbeddingProvider};
use reqwest::Client;
use serde::Deserialize;
use tracing::instrument;

use crate::http_retry::with_retry;
use crate::http_status::map_embedding_http_status;

/// Provider d'embeddings via l'API Ollama (`/api/embed`, doc officielle).
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
        let mut body = serde_json::json!({
            "model": self.model,
            "input": text,
        });
        if let Some(dim) = self.dimension {
            body["dimensions"] = serde_json::json!(dim);
        }
        let url = self.url("/api/embed");
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
        .map_err(|e| match e {
            crate::http_retry::HttpRetryError::CircuitOpen(c) => EmbeddingError::Unavailable {
                provider: "ollama".into(),
                message: format!("circuit ouvert ({})", c.retry_after_secs),
            },
            crate::http_retry::HttpRetryError::Request(err) => EmbeddingError::Network {
                provider: "ollama".into(),
                message: err.to_string(),
                source: Some(Box::new(err)),
            },
        })?;

        if !response.status().is_success() {
            return Err(map_embedding_http_status("ollama", response.status()));
        }

        let parsed: OllamaEmbeddingResponse =
            response
                .json()
                .await
                .map_err(|e| EmbeddingError::InvalidResponse {
                    provider: "ollama".into(),
                    message: e.to_string(),
                })?;

        let vector = parsed
            .embeddings
            .into_iter()
            .next()
            .ok_or_else(|| EmbeddingError::InvalidResponse {
                provider: "ollama".into(),
                message: "réponse embed sans vecteur".into(),
            })?;

        if let Some(expected) = self.dimension {
            if vector.len() != expected {
                return Err(EmbeddingError::InvalidResponse {
                    provider: "ollama".into(),
                    message: format!(
                        "dimension embedding {} != attendue {expected}",
                        vector.len()
                    ),
                });
            }
        }

        Ok(vector)
    }
}

#[derive(Debug, Deserialize)]
struct OllamaEmbeddingResponse {
    embeddings: Vec<Vec<f32>>,
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
