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

/// Provider LLM OpenAI-compatible (`/chat/completions`).
pub struct OpenAiCompatibleLlmProvider {
    name: &'static str,
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
    timeout: Duration,
    max_retries: u32,
    last_usage: Mutex<Option<LlmUsageRecorded>>,
}

impl OpenAiCompatibleLlmProvider {
    /// Crée un provider OpenAI-compatible.
    #[must_use]
    pub fn new(
        name: &'static str,
        client: Client,
        api_key: impl Into<String>,
        model: impl Into<String>,
        base_url: impl Into<String>,
        timeout_secs: u64,
        max_retries: u32,
    ) -> Self {
        Self {
            name,
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
        format!("{}/chat/completions", self.base_url)
    }

    fn record_usage(&self, operation: &str, usage: Option<&OpenAiUsage>) {
        let recorded = LlmUsageRecorded {
            provider: self.name.to_string(),
            operation: operation.into(),
            prompt_tokens: usage.and_then(|u| u.prompt_tokens),
            completion_tokens: usage.and_then(|u| u.completion_tokens),
        };
        if let Ok(mut guard) = self.last_usage.lock() {
            *guard = Some(recorded);
        }
    }

    fn parse_memory_draft(content: &str, provider: &str) -> Result<MemoryDraft, LlmError> {
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
            provider: provider.into(),
            message: e.to_string(),
        })
    }

    async fn post_chat(&self, body: serde_json::Value) -> Result<OpenAiChatResponse, LlmError> {
        let started = Instant::now();
        let client = self.client.clone();
        let api_key = self.api_key.clone();
        let url = self.endpoint();
        let name = self.name;
        let max_retries = self.max_retries;
        let timeout = self.timeout;

        let response = tokio::time::timeout(timeout, async {
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
        .map_err(|_| LlmError::Unavailable {
            provider: name.into(),
            message: "timeout".into(),
        })?
        .map_err(|e| match e {
            crate::http_retry::HttpRetryError::CircuitOpen(c) => LlmError::Unavailable {
                provider: name.into(),
                message: format!("circuit ouvert ({})", c.retry_after_secs),
            },
            crate::http_retry::HttpRetryError::Request(err) => LlmError::ProviderError {
                provider: name.into(),
                message: err.to_string(),
            },
        })?;

        if !response.status().is_success() {
            return Err(map_llm_http_status(name, response.status()));
        }

        let parsed: OpenAiChatResponse =
            response.json().await.map_err(|e| LlmError::ProviderError {
                provider: name.into(),
                message: e.to_string(),
            })?;

        if let Some(usage) = &parsed.usage {
            tracing::info!(
                provider = name,
                prompt_tokens = usage.prompt_tokens,
                completion_tokens = usage.completion_tokens,
                latency_ms = started.elapsed().as_millis(),
                "tokens LLM"
            );
        }

        Ok(parsed)
    }
}

#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleLlmProvider {
    fn name(&self) -> &'static str {
        self.name
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
        let parsed = self.post_chat(body).await?;
        self.record_usage("generate_memory_draft", parsed.usage.as_ref());
        let content = parsed
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| LlmError::ProviderError {
                provider: self.name.into(),
                message: "réponse vide".into(),
            })?;
        Self::parse_memory_draft(&content, self.name)
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
        let parsed = self.post_chat(body).await?;
        self.record_usage("chat", parsed.usage.as_ref());
        parsed
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| LlmError::ProviderError {
                provider: self.name.into(),
                message: "réponse vide".into(),
            })
    }

    fn last_usage(&self) -> Option<LlmUsageRecorded> {
        self.last_usage.lock().ok().and_then(|g| g.clone())
    }
}