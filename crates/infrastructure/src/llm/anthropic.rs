use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use orchestrator::{
    ChatMessage, LlmCapabilities, LlmError, LlmProvider, LlmUsageRecorded, MemoryDraft,
};
use reqwest::Client;
use serde::Deserialize;
use tracing::instrument;

use crate::http_retry::with_retry;
use crate::http_status::map_llm_http_status;

/// Provider Anthropic Claude (API Messages).
pub struct AnthropicLlmProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
    timeout: Duration,
    max_retries: u32,
    last_usage: Mutex<Option<LlmUsageRecorded>>,
}

impl AnthropicLlmProvider {
    /// Crée un provider Anthropic.
    #[must_use]
    pub fn new(
        client: Client,
        api_key: impl Into<String>,
        model: impl Into<String>,
        base_url: impl Into<String>,
        timeout_secs: u64,
        max_retries: u32,
    ) -> Self {
        Self {
            client,
            api_key: api_key.into(),
            model: model.into(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            timeout: Duration::from_secs(timeout_secs),
            max_retries,
            last_usage: Mutex::new(None),
        }
    }

    fn endpoint(&self) -> String {
        format!("{}/v1/messages", self.base_url)
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
            provider: "anthropic".into(),
            message: e.to_string(),
        })
    }

    async fn post_messages(&self, body: serde_json::Value) -> Result<AnthropicResponse, LlmError> {
        let client = self.client.clone();
        let api_key = self.api_key.clone();
        let url = self.endpoint();
        let max_retries = self.max_retries;
        let timeout = self.timeout;

        let response = tokio::time::timeout(timeout, async {
            with_retry("anthropic", max_retries, || {
                let client = client.clone();
                let body = body.clone();
                let api_key = api_key.clone();
                let url = url.clone();
                async move {
                    client
                        .post(url)
                        .header("x-api-key", api_key)
                        .header("anthropic-version", "2023-06-01")
                        .json(&body)
                        .send()
                        .await
                }
            })
            .await
        })
        .await
        .map_err(|_| LlmError::Unavailable {
            provider: "anthropic".into(),
            message: "timeout".into(),
        })?
        .map_err(|e| match e {
            crate::http_retry::HttpRetryError::CircuitOpen(c) => LlmError::Unavailable {
                provider: "anthropic".into(),
                message: format!("circuit ouvert ({})", c.retry_after_secs),
            },
            crate::http_retry::HttpRetryError::Request(err) => LlmError::ProviderError {
                provider: "anthropic".into(),
                message: err.to_string(),
            },
        })?;

        if !response.status().is_success() {
            return Err(map_llm_http_status("anthropic", response.status()));
        }

        response.json().await.map_err(|e| LlmError::ProviderError {
            provider: "anthropic".into(),
            message: e.to_string(),
        })
    }

    fn extract_text(response: &AnthropicResponse) -> Result<String, LlmError> {
        for block in &response.content {
            if block.r#type == "text" {
                return Ok(block.text.clone());
            }
        }
        Err(LlmError::ProviderError {
            provider: "anthropic".into(),
            message: "réponse vide".into(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContentBlock>,
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContentBlock {
    r#type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
}

#[async_trait]
impl LlmProvider for AnthropicLlmProvider {
    fn name(&self) -> &'static str {
        "anthropic"
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
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 4096,
            "system": system_prompt,
            "messages": [{"role": "user", "content": user_prompt}],
        });
        let parsed = self.post_messages(body).await?;
        if let Ok(mut guard) = self.last_usage.lock() {
            *guard = Some(LlmUsageRecorded {
                provider: "anthropic".into(),
                operation: "generate_memory_draft".into(),
                prompt_tokens: parsed.usage.as_ref().and_then(|u| u.input_tokens),
                completion_tokens: parsed.usage.as_ref().and_then(|u| u.output_tokens),
            });
        }
        let content = Self::extract_text(&parsed)?;
        Self::parse_memory_draft(&content)
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<String, LlmError> {
        let mut system = String::new();
        let mut anthropic_messages = Vec::new();
        for msg in messages {
            if msg.role == "system" {
                system = msg.content.clone();
            } else {
                anthropic_messages.push(serde_json::json!({
                    "role": msg.role,
                    "content": msg.content,
                }));
            }
        }
        let mut body = serde_json::json!({
            "model": self.model,
            "max_tokens": 4096,
            "messages": anthropic_messages,
        });
        if !system.is_empty() {
            body["system"] = serde_json::Value::String(system);
        }
        let parsed = self.post_messages(body).await?;
        Self::extract_text(&parsed)
    }

    fn last_usage(&self) -> Option<LlmUsageRecorded> {
        self.last_usage.lock().ok().and_then(|g| g.clone())
    }
}