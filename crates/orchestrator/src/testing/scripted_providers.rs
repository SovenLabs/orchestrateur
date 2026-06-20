use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use cortex::{Embedding, EmbeddingCapabilities, EmbeddingError, EmbeddingProvider};

use crate::llm::{ChatMessage, LlmCapabilities, LlmError, LlmProvider, LlmUsageRecorded};
use crate::memory_draft::MemoryDraft;

/// Provider LLM scripté — file d'erreurs/succès pour tests de résilience.
pub struct ScriptedLlmProvider {
    provider_name: &'static str,
    script: Mutex<VecDeque<Result<MemoryDraft, LlmError>>>,
    calls: AtomicUsize,
    last_usage: Mutex<Option<LlmUsageRecorded>>,
}

impl ScriptedLlmProvider {
    /// Crée un provider nommé avec un script de réponses.
    #[must_use]
    pub fn new(provider_name: &'static str, script: Vec<Result<MemoryDraft, LlmError>>) -> Arc<Self> {
        Arc::new(Self {
            provider_name,
            script: Mutex::new(script.into()),
            calls: AtomicUsize::new(0),
            last_usage: Mutex::new(None),
        })
    }

    /// Simule une erreur 429 puis un succès (intensité 1 — résilience).
    #[must_use]
    pub fn xai_fail_429_then_ok(success: MemoryDraft) -> Arc<Self> {
        Self::new(
            "xai",
            vec![
                Err(LlmError::RateLimited {
                    provider: "xai".into(),
                }),
                Ok(success),
            ],
        )
    }

    /// Nombre d'appels `generate_memory_draft` effectués.
    pub fn call_count(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl LlmProvider for ScriptedLlmProvider {
    fn name(&self) -> &'static str {
        self.provider_name
    }

    fn capabilities(&self) -> LlmCapabilities {
        LlmCapabilities {
            supports_structured_output: true,
            ..Default::default()
        }
    }

    async fn generate_memory_draft(
        &self,
        _system: &str,
        _user: &str,
    ) -> Result<MemoryDraft, LlmError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let next = self
            .script
            .lock()
            .map_err(|e| LlmError::ProviderError {
                provider: self.provider_name.into(),
                message: e.to_string(),
            })?
            .pop_front()
            .unwrap_or_else(|| {
                Err(LlmError::ProviderError {
                    provider: self.provider_name.into(),
                    message: "script épuisé".into(),
                })
            });
        if next.is_ok() {
            if let Ok(mut guard) = self.last_usage.lock() {
                *guard = Some(LlmUsageRecorded {
                    provider: self.provider_name.into(),
                    operation: "generate_memory_draft".into(),
                    prompt_tokens: Some(10),
                    completion_tokens: Some(20),
                });
            }
        }
        next
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<String, LlmError> {
        Ok(messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default())
    }

    fn last_usage(&self) -> Option<LlmUsageRecorded> {
        self.last_usage.lock().ok().and_then(|g| g.clone())
    }
}

/// Provider Ollama mock stable (succès systématique).
pub struct StableOllamaLlmProvider;

#[async_trait]
impl LlmProvider for StableOllamaLlmProvider {
    fn name(&self) -> &'static str {
        "ollama"
    }

    fn capabilities(&self) -> LlmCapabilities {
        LlmCapabilities::default()
    }

    async fn generate_memory_draft(
        &self,
        _system: &str,
        user: &str,
    ) -> Result<MemoryDraft, LlmError> {
        Ok(MemoryDraft {
            title: "Ollama fallback".into(),
            content: user.into(),
            tags: vec![],
            backlinks: vec![],
        })
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<String, LlmError> {
        Ok(messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default())
    }
}

/// Embeddings déterministes avec compteur d'appels batch/single.
pub struct CountingEmbeddingProvider {
    inner: super::InMemoryEmbeddingProvider,
    embed_calls: AtomicUsize,
    batch_calls: AtomicUsize,
}

impl CountingEmbeddingProvider {
    /// Crée un compteur autour du mock in-memory.
    #[must_use]
    pub fn new(dim: usize) -> Arc<Self> {
        Arc::new(Self {
            inner: super::InMemoryEmbeddingProvider::new(dim),
            embed_calls: AtomicUsize::new(0),
            batch_calls: AtomicUsize::new(0),
        })
    }

    /// Appels à `embed`.
    pub fn embed_calls(&self) -> usize {
        self.embed_calls.load(Ordering::SeqCst)
    }

    /// Appels à `embed_batch`.
    pub fn batch_calls(&self) -> usize {
        self.batch_calls.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl EmbeddingProvider for CountingEmbeddingProvider {
    fn name(&self) -> &'static str {
        "counting"
    }

    fn capabilities(&self) -> EmbeddingCapabilities {
        self.inner.capabilities()
    }

    async fn embed(&self, text: &str) -> Result<Embedding, EmbeddingError> {
        self.embed_calls.fetch_add(1, Ordering::SeqCst);
        self.inner.embed(text).await
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbeddingError> {
        self.batch_calls.fetch_add(1, Ordering::SeqCst);
        self.inner.embed_batch(texts).await
    }
}

/// Vector store qui échoue un nombre configurable de fois sur `upsert`.
pub struct FailNthVectorStore {
    inner: Arc<dyn cortex::VectorStore>,
    failures_remaining: AtomicUsize,
}

impl FailNthVectorStore {
    /// Enveloppe un store et échoue sur les N premiers `upsert`.
    #[must_use]
    pub fn new(inner: Arc<dyn cortex::VectorStore>, fail_count: usize) -> Arc<Self> {
        Arc::new(Self {
            inner,
            failures_remaining: AtomicUsize::new(fail_count),
        })
    }
}

#[async_trait]
impl cortex::VectorStore for FailNthVectorStore {
    async fn upsert(
        &self,
        memory_id: cortex::MemoryId,
        embedding: &[f32],
    ) -> Result<(), cortex::CortexError> {
        let remaining = self.failures_remaining.load(Ordering::SeqCst);
        if remaining > 0 {
            self.failures_remaining.fetch_sub(1, Ordering::SeqCst);
            return Err(cortex::CortexError::GraphError(
                "upsert simulé en échec".into(),
            ));
        }
        self.inner.upsert(memory_id, embedding).await
    }

    async fn semantic_search(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<cortex::SearchHit>, cortex::CortexError> {
        self.inner.semantic_search(query_embedding, limit).await
    }

    async fn hybrid_search(
        &self,
        query_embedding: &[f32],
        filter: &cortex::SearchFilter,
    ) -> Result<Vec<cortex::SearchHit>, cortex::CortexError> {
        self.inner.hybrid_search(query_embedding, filter).await
    }

    async fn get_embedding(
        &self,
        memory_id: cortex::MemoryId,
    ) -> Result<Option<Vec<f32>>, cortex::CortexError> {
        self.inner.get_embedding(memory_id).await
    }

    async fn delete(&self, memory_id: cortex::MemoryId) -> Result<(), cortex::CortexError> {
        self.inner.delete(memory_id).await
    }
}

/// LLM retournant du JSON invalide (structured output cassé).
pub struct InvalidJsonLlmProvider;

#[async_trait]
impl LlmProvider for InvalidJsonLlmProvider {
    fn name(&self) -> &'static str {
        "invalid-json"
    }

    fn capabilities(&self) -> LlmCapabilities {
        LlmCapabilities::default()
    }

    async fn generate_memory_draft(
        &self,
        _system: &str,
        _user: &str,
    ) -> Result<MemoryDraft, LlmError> {
        Err(LlmError::StructuredOutputInvalid {
            provider: "invalid-json".into(),
            message: "JSON malformé simulé".into(),
        })
    }

    async fn chat(&self, _messages: &[ChatMessage]) -> Result<String, LlmError> {
        Err(LlmError::StructuredOutputInvalid {
            provider: "invalid-json".into(),
            message: "JSON malformé simulé".into(),
        })
    }
}