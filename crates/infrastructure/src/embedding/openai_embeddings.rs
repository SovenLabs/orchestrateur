use std::time::Duration;

use async_trait::async_trait;
use cortex::{Embedding, EmbeddingCapabilities, EmbeddingError, EmbeddingProvider};
use reqwest::Client;
use serde::Deserialize;
use tracing::instrument;

use crate::http_retry::with_retry;

/// Provider embeddings OpenAI-compatible (`/embeddings`).
pub struct OpenAiEmbeddingsProvider {
    name: &'static str,
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
    timeout: Duration,
    max_retries: u32,
    expected_dim: Option<usize>,
}

impl OpenAiEmbeddingsProvider {
    /// Crée un provider embeddings OpenAI-compatible.
    #[must_use]
    pub fn new(
        name: &'static str,
        client: Client,
        api_key: impl Into<String>,
        model: impl Into<String>,
        base_url: impl Into<String>,
        timeout_secs: u64,
        max_retries: u32,
        expected_dim: Option<usize>,
    ) -> Self {
        Self {
            name,
            client,
            api_key: api_key.into(),
            model: model.into(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            timeout: Duration::from_secs(timeout_secs),
            max_retries,
            expected_dim,
        }
    }

    fn endpoint(&self) -> String {
        format!("{}/embeddings", self.base_url)
    }
}

#[derive(Debug, Deserialize)]
struct EmbeddingsResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

#[async_trait]
impl EmbeddingProvider for OpenAiEmbeddingsProvider {
    fn name(&self) -> &str {
        self.name
    }

    fn capabilities(&self) -> EmbeddingCapabilities {
        EmbeddingCapabilities {
            supports_batch: true,
            max_batch_size: Some(64),
            typical_dimensions: self.expected_dim,
            ..Default::default()
        }
    }

    #[instrument(skip(self, text))]
    async fn embed(&self, text: &str) -> Result<Embedding, EmbeddingError> {
        let client = self.client.clone();
        let api_key = self.api_key.clone();
        let url = self.endpoint();
        let model = self.model.clone();
        let name = self.name;
        let max_retries = self.max_retries;
        let body = serde_json::json!({ "model": model, "input": text });

        let response = tokio::time::timeout(self.timeout, async {
            with_retry(name, max_retries, || {
                let client = client.clone();
                let body = body.clone();
                let api_key = api_key.clone();
                let url = url.clone();
                async move {
                    client
                        .post(url)
                        .bearer_auth(api_key)
                        .json(&body)
                        .send()
                        .await
                }
            })
            .await
        })
        .await
        .map_err(|_| EmbeddingError::Unavailable {
            provider: name.into(),
            message: "timeout".into(),
        })?
        .map_err(|e| EmbeddingError::Network {
            provider: name.into(),
            message: e.to_string(),
            source: None,
        })?;

        if !response.status().is_success() {
            return Err(EmbeddingError::InvalidResponse {
                provider: name.into(),
                message: format!("HTTP {}", response.status()),
            });
        }

        let parsed: EmbeddingsResponse = response
            .json()
            .await
            .map_err(|e| EmbeddingError::InvalidResponse {
                provider: name.into(),
                message: e.to_string(),
            })?;

        let vector = parsed
            .data
            .first()
            .map(|d| d.embedding.clone())
            .ok_or_else(|| EmbeddingError::InvalidResponse {
                provider: name.into(),
                message: "réponse vide".into(),
            })?;

        if let Some(expected) = self.expected_dim {
            if vector.len() != expected {
                return Err(EmbeddingError::InvalidResponse {
                    provider: name.into(),
                    message: format!(
                        "dimension {} != attendue {}",
                        vector.len(),
                        expected
                    ),
                });
            }
        }

        Ok(Embedding::new(vector))
    }
}