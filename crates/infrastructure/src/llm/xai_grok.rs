use std::sync::Mutex;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use orchestrator::{
    ChatMessage, LlmCapabilities, LlmError, LlmProvider, LlmUsageRecorded, MemoryDraft,
};
use reqwest::Client;
use serde::Deserialize;
use tracing::instrument;

use crate::http_retry::with_retry;
use crate::http_status::map_llm_http_status;

const XAI_API_URL: &str = "https://api.x.ai/v1/chat/completions";

/// Provider xAI Grok avec Structured Outputs (JSON mode).
pub struct XaiGrokProvider {
    client: Client,
    api_key: String,
    model: String,
    timeout: Duration,
    max_retries: u32,
    last_usage: Mutex<Option<LlmUsageRecorded>>,
}

impl XaiGrokProvider {
    /// Crée un provider xAI. La clé API ne doit jamais être loggée.
    #[must_use]
    pub fn new(
        client: Client,
        api_key: impl Into<String>,
        model: impl Into<String>,
        timeout_secs: u64,
        max_retries: u32,
    ) -> Self {
        Self {
            client,
            api_key: api_key.into(),
            model: model.into(),
            timeout: Duration::from_secs(timeout_secs),
            max_retries,
            last_usage: Mutex::new(None),
        }
    }

    fn record_usage(&self, operation: &str, usage: Option<&XaiUsage>) {
        let recorded = LlmUsageRecorded {
            provider: "xai".into(),
            operation: operation.into(),
            prompt_tokens: usage.and_then(|u| u.prompt_tokens),
            completion_tokens: usage.and_then(|u| u.completion_tokens),
        };
        if let Ok(mut guard) = self.last_usage.lock() {
            *guard = Some(recorded);
        }
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
            provider: "xai".into(),
            message: e.to_string(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct XaiChatResponse {
    choices: Vec<XaiChoice>,
    usage: Option<XaiUsage>,
}

#[derive(Debug, Deserialize)]
struct XaiChoice {
    message: XaiMessage,
}

#[derive(Debug, Deserialize)]
struct XaiMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct XaiUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
}

#[async_trait]
impl LlmProvider for XaiGrokProvider {
    fn name(&self) -> &'static str {
        "xai"
    }

    fn capabilities(&self) -> LlmCapabilities {
        LlmCapabilities {
            supports_structured_output: true,
            supports_streaming: false,
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
            "response_format": { "type": "json_object" },
            "temperature": 0.2,
        });

        let started = Instant::now();
        let client = self.client.clone();
        let api_key = self.api_key.clone();
        let timeout = self.timeout;
        let max_retries = self.max_retries;

        let response = tokio::time::timeout(timeout, async {
            with_retry("xai", max_retries, || {
                let client = client.clone();
                let body = body.clone();
                let api_key = api_key.clone();
                async move {
                    client
                        .post(XAI_API_URL)
                        .bearer_auth(api_key)
                        .json(&body)
                        .send()
                        .await
                }
            })
            .await
        })
        .await
        .map_err(|_| LlmError::Unavailable {
            provider: "xai".into(),
            message: "timeout".into(),
        })?
        .map_err(|e| match e {
            crate::http_retry::HttpRetryError::CircuitOpen(c) => LlmError::Unavailable {
                provider: "xai".into(),
                message: format!("circuit ouvert ({})", c.retry_after_secs),
            },
            crate::http_retry::HttpRetryError::Request(err) => LlmError::ProviderError {
                provider: "xai".into(),
                message: err.to_string(),
            },
        })?;

        if !response.status().is_success() {
            return Err(map_llm_http_status("xai", response.status()));
        }

        let parsed: XaiChatResponse =
            response.json().await.map_err(|e| LlmError::ProviderError {
                provider: "xai".into(),
                message: e.to_string(),
            })?;

        self.record_usage("generate_memory_draft", parsed.usage.as_ref());
        if let Some(usage) = &parsed.usage {
            tracing::info!(
                provider = "xai",
                prompt_tokens = usage.prompt_tokens,
                completion_tokens = usage.completion_tokens,
                latency_ms = started.elapsed().as_millis(),
                "tokens xAI"
            );
        }

        let content = parsed
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| LlmError::ProviderError {
                provider: "xai".into(),
                message: "réponse vide".into(),
            })?;

        Self::parse_memory_draft(&content)
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<String, LlmError> {
        let msgs: Vec<_> = messages
            .iter()
            .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
            .collect();
        let body = serde_json::json!({
            "model": self.model,
            "messages": msgs,
        });
        let client = self.client.clone();
        let api_key = self.api_key.clone();
        let max_retries = self.max_retries;
        let response = tokio::time::timeout(self.timeout, async {
            with_retry("xai", max_retries, || {
                let client = client.clone();
                let body = body.clone();
                let api_key = api_key.clone();
                async move {
                    client
                        .post(XAI_API_URL)
                        .bearer_auth(api_key)
                        .json(&body)
                        .send()
                        .await
                }
            })
            .await
        })
        .await
        .map_err(|_| LlmError::Unavailable {
            provider: "xai".into(),
            message: "timeout".into(),
        })?
        .map_err(|e| match e {
            crate::http_retry::HttpRetryError::CircuitOpen(c) => LlmError::Unavailable {
                provider: "xai".into(),
                message: format!("circuit ouvert ({})", c.retry_after_secs),
            },
            crate::http_retry::HttpRetryError::Request(err) => LlmError::ProviderError {
                provider: "xai".into(),
                message: err.to_string(),
            },
        })?;

        if !response.status().is_success() {
            return Err(map_llm_http_status("xai", response.status()));
        }

        let parsed: XaiChatResponse =
            response.json().await.map_err(|e| LlmError::ProviderError {
                provider: "xai".into(),
                message: e.to_string(),
            })?;

        self.record_usage("chat", parsed.usage.as_ref());

        parsed
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| LlmError::ProviderError {
                provider: "xai".into(),
                message: "réponse vide".into(),
            })
    }

    fn last_usage(&self) -> Option<LlmUsageRecorded> {
        self.last_usage.lock().ok().and_then(|g| g.clone())
    }
}
