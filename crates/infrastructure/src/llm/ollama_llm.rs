use std::time::{Duration, Instant};

use async_trait::async_trait;
use orchestrator::{ChatMessage, LlmCapabilities, LlmError, LlmProvider, MemoryDraft};
use reqwest::Client;
use serde::Deserialize;
use tracing::instrument;

use crate::http_retry::with_retry;

/// Provider LLM Ollama (fallback local, JSON mode).
pub struct OllamaLlmProvider {
    client: Client,
    base_url: String,
    model: String,
    timeout: Duration,
    max_retries: u32,
}

impl OllamaLlmProvider {
    /// Crée un provider Ollama chat.
    #[must_use]
    pub fn new(
        client: Client,
        base_url: impl Into<String>,
        model: impl Into<String>,
        timeout_secs: u64,
        max_retries: u32,
    ) -> Self {
        Self {
            client,
            base_url: base_url.into(),
            model: model.into(),
            timeout: Duration::from_secs(timeout_secs),
            max_retries,
        }
    }

    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn parse_memory_draft(content: &str) -> Result<MemoryDraft, LlmError> {
        let trimmed = content.trim();
        let json_str = if trimmed.starts_with("```") {
            trimmed
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim()
        } else {
            trimmed
        };
        serde_json::from_str(json_str).map_err(|e| LlmError::StructuredOutputInvalid {
            provider: "ollama".into(),
            message: e.to_string(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessage,
}

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    content: String,
}

#[async_trait]
impl LlmProvider for OllamaLlmProvider {
    fn name(&self) -> &'static str {
        "ollama"
    }

    fn capabilities(&self) -> LlmCapabilities {
        LlmCapabilities {
            supports_structured_output: true,
            ..Default::default()
        }
    }

    #[instrument(skip(self, system_prompt, user_prompt))]
    async fn generate_memory_draft(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<MemoryDraft, LlmError> {
        let messages = vec![
            serde_json::json!({"role": "system", "content": system_prompt}),
            serde_json::json!({"role": "user", "content": user_prompt}),
        ];
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "format": "json",
            "stream": false,
        });
        let url = self.url("/api/chat");
        let started = Instant::now();
        let client = self.client.clone();
        let max_retries = self.max_retries;

        let response = tokio::time::timeout(self.timeout, async {
            with_retry("ollama", max_retries, || {
                let client = client.clone();
                let body = body.clone();
                let url = url.clone();
                async move { client.post(&url).json(&body).send().await }
            })
            .await
        })
        .await
        .map_err(|_| LlmError::Unavailable {
            provider: "ollama".into(),
            message: "timeout".into(),
        })?
        .map_err(|e| LlmError::ProviderError {
            provider: "ollama".into(),
            message: e.to_string(),
        })?;

        if !response.status().is_success() {
            return Err(LlmError::Unavailable {
                provider: "ollama".into(),
                message: format!("HTTP {}", response.status()),
            });
        }

        let parsed: OllamaChatResponse = response.json().await.map_err(|e| LlmError::ProviderError {
            provider: "ollama".into(),
            message: e.to_string(),
        })?;

        tracing::info!(
            provider = "ollama",
            latency_ms = started.elapsed().as_millis(),
            "réponse Ollama"
        );

        Self::parse_memory_draft(&parsed.message.content)
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<String, LlmError> {
        let msgs: Vec<_> = messages
            .iter()
            .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
            .collect();
        let body = serde_json::json!({
            "model": self.model,
            "messages": msgs,
            "stream": false,
        });
        let url = self.url("/api/chat");
        let response = tokio::time::timeout(self.timeout, self.client.post(&url).json(&body).send())
            .await
            .map_err(|_| LlmError::Unavailable {
                provider: "ollama".into(),
                message: "timeout".into(),
            })?
            .map_err(|e| LlmError::ProviderError {
                provider: "ollama".into(),
                message: e.to_string(),
            })?;

        let parsed: OllamaChatResponse = response.json().await.map_err(|e| LlmError::ProviderError {
            provider: "ollama".into(),
            message: e.to_string(),
        })?;
        Ok(parsed.message.content)
    }
}