use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use cortex::{
    cosine_similarity, CortexError, Embedding, EmbeddingCapabilities, EmbeddingError,
    EmbeddingProvider, Memory, MemoryId, MemoryRepository, SearchFilter, SearchHit,
    VectorStore,
};

use crate::llm::{LlmCapabilities, LlmError, LlmProvider};
use crate::memory_draft::MemoryDraft;

use crate::config::OrchestratorConfig;
use crate::deps::AppDependencies;
use crate::draft::{DraftError, DraftRepository, DraftStatus, StoredDraft};
use crate::events::NoopEventPublisher;
use super::session_mock::InMemorySessionRepository;

/// Bundle prêt à l'emploi : les trois mocks + configuration par défaut.
pub struct MockBundle {
    /// Mock de persistance mémoires.
    pub memory_repo: Arc<InMemoryMemoryRepository>,
    /// Mock de vector store.
    pub vector_store: Arc<InMemoryVectorStore>,
    /// Mock d'embeddings déterministes.
    pub embedding: Arc<InMemoryEmbeddingProvider>,
    /// Mock LLM déterministe.
    pub llm: Arc<InMemoryLlmProvider>,
    /// Mock sessions agent.
    pub session_repo: Arc<InMemorySessionRepository>,
    /// Mock file de brouillons.
    pub draft_repo: Arc<InMemoryDraftRepository>,
    /// Configuration de test.
    pub config: OrchestratorConfig,
}

impl MockBundle {
    /// Crée un bundle de mocks thread-safe avec configuration par défaut.
    #[must_use]
    pub fn new() -> Self {
        let config = OrchestratorConfig::default();
        let embedding = Arc::new(InMemoryEmbeddingProvider::new(config.embedding_dim));
        let llm = Arc::new(InMemoryLlmProvider);
        Self {
            memory_repo: Arc::new(InMemoryMemoryRepository::new()),
            vector_store: Arc::new(InMemoryVectorStore::new()),
            embedding,
            llm,
            session_repo: Arc::new(InMemorySessionRepository::new()),
            draft_repo: Arc::new(InMemoryDraftRepository::new()),
            config,
        }
    }

    /// Convertit le bundle en [`AppDependencies`] (publisher noop pour les tests).
    #[must_use]
    pub fn into_deps(self) -> AppDependencies {
        AppDependencies::for_tests(
            self.memory_repo,
            self.vector_store,
            self.embedding,
            self.llm,
            self.session_repo,
            self.draft_repo,
            self.config,
            Arc::new(NoopEventPublisher),
        )
    }
}

/// Persistance brouillons en mémoire (`HashMap` + `RwLock`).
pub struct InMemoryDraftRepository {
    inner: RwLock<HashMap<String, StoredDraft>>,
}

impl InMemoryDraftRepository {
    /// Crée un dépôt vide.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryDraftRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DraftRepository for InMemoryDraftRepository {
    async fn save(&self, stored: &StoredDraft) -> Result<(), DraftError> {
        let mut guard = self
            .inner
            .write()
            .map_err(|e| DraftError::Io(e.to_string()))?;
        guard.insert(stored.id.clone(), stored.clone());
        Ok(())
    }

    async fn get_by_id(&self, id: &str) -> Result<StoredDraft, DraftError> {
        let guard = self
            .inner
            .read()
            .map_err(|e| DraftError::Io(e.to_string()))?;
        guard
            .get(id)
            .cloned()
            .ok_or_else(|| DraftError::NotFound(id.to_string()))
    }

    async fn list(&self, status: Option<DraftStatus>) -> Result<Vec<StoredDraft>, DraftError> {
        let guard = self
            .inner
            .read()
            .map_err(|e| DraftError::Io(e.to_string()))?;
        let mut items: Vec<StoredDraft> = guard
            .values()
            .filter(|d| status.is_none_or(|s| d.status == s))
            .cloned()
            .collect();
        items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(items)
    }

    async fn update_status(
        &self,
        id: &str,
        status: DraftStatus,
    ) -> Result<StoredDraft, DraftError> {
        let mut guard = self
            .inner
            .write()
            .map_err(|e| DraftError::Io(e.to_string()))?;
        let stored = guard
            .get_mut(id)
            .ok_or_else(|| DraftError::NotFound(id.to_string()))?;
        stored.status = status;
        Ok(stored.clone())
    }
}

impl Default for MockBundle {
    fn default() -> Self {
        Self::new()
    }
}

/// Persistance mémoires en mémoire (`HashMap` + `RwLock`).
pub struct InMemoryMemoryRepository {
    inner: RwLock<HashMap<MemoryId, Memory>>,
}

impl InMemoryMemoryRepository {
    /// Crée un dépôt vide.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    /// Nombre de mémoires stockées (utilitaire de test).
    pub fn len(&self) -> usize {
        self.inner.read().map_or(0, |m| m.len())
    }

    /// Indique si le dépôt est vide (utilitaire de test).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for InMemoryMemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MemoryRepository for InMemoryMemoryRepository {
    async fn save(&self, memory: &Memory) -> Result<(), CortexError> {
        let mut guard = self
            .inner
            .write()
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
        guard.insert(memory.id, memory.clone());
        Ok(())
    }

    async fn get_by_id(&self, id: MemoryId) -> Result<Memory, CortexError> {
        let guard = self
            .inner
            .read()
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
        guard
            .get(&id)
            .cloned()
            .ok_or(CortexError::MemoryNotFound(id))
    }

    async fn list(&self) -> Result<Vec<Memory>, CortexError> {
        let guard = self
            .inner
            .read()
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
        Ok(guard.values().cloned().collect())
    }

    async fn delete(&self, id: MemoryId) -> Result<(), CortexError> {
        let mut guard = self
            .inner
            .write()
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
        guard.remove(&id).ok_or(CortexError::MemoryNotFound(id))?;
        Ok(())
    }
}

/// Vector store en mémoire avec recherche par similarité cosinus.
pub struct InMemoryVectorStore {
    inner: RwLock<HashMap<MemoryId, Vec<f32>>>,
}

impl InMemoryVectorStore {
    /// Crée un index vectoriel vide.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    /// Nombre de vecteurs indexés (utilitaire de test).
    pub fn len(&self) -> usize {
        self.inner.read().map_or(0, |m| m.len())
    }

    /// Indique si l'index est vide (utilitaire de test).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for InMemoryVectorStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn upsert(&self, memory_id: MemoryId, embedding: &[f32]) -> Result<(), CortexError> {
        let mut guard = self
            .inner
            .write()
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
        guard.insert(memory_id, embedding.to_vec());
        Ok(())
    }

    async fn semantic_search(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchHit>, CortexError> {
        let guard = self
            .inner
            .read()
            .map_err(|e| CortexError::GraphError(e.to_string()))?;

        let mut hits: Vec<SearchHit> = guard
            .iter()
            .filter_map(|(id, vec)| {
                let score = cosine_similarity(query_embedding, vec)?;
                Some(SearchHit {
                    memory_id: *id,
                    score,
                    snippet: None,
                })
            })
            .collect();

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(limit);
        Ok(hits)
    }

    async fn hybrid_search(
        &self,
        query_embedding: &[f32],
        filter: &SearchFilter,
    ) -> Result<Vec<SearchHit>, CortexError> {
        let candidate_limit = filter.limit.unwrap_or(256);
        let mut hits = self
            .semantic_search(query_embedding, candidate_limit)
            .await?;

        if let Some(min) = filter.min_score {
            hits.retain(|h| h.score >= min);
        }

        Ok(hits)
    }

    async fn get_embedding(&self, memory_id: MemoryId) -> Result<Option<Vec<f32>>, CortexError> {
        let guard = self
            .inner
            .read()
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
        Ok(guard.get(&memory_id).cloned())
    }

    async fn delete(&self, memory_id: MemoryId) -> Result<(), CortexError> {
        let mut guard = self
            .inner
            .write()
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
        guard
            .remove(&memory_id)
            .ok_or(CortexError::MemoryNotFound(memory_id))?;
        Ok(())
    }
}

/// Embeddings déterministes dérivés du hash du texte (reproductibles en test).
pub struct InMemoryEmbeddingProvider {
    dim: usize,
}

impl InMemoryEmbeddingProvider {
    /// Crée un provider avec la dimension indiquée.
    #[must_use]
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }

    fn deterministic_vector(&self, text: &str) -> Vec<f32> {
        let mut vec = vec![0.0_f32; self.dim];
        for (i, byte) in text.bytes().enumerate() {
            vec[i % self.dim] += f32::from(byte) / 255.0;
        }
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vec {
                *v /= norm;
            }
        }
        vec
    }
}

#[async_trait]
impl EmbeddingProvider for InMemoryEmbeddingProvider {
    fn name(&self) -> &'static str {
        "in-memory"
    }

    fn capabilities(&self) -> EmbeddingCapabilities {
        EmbeddingCapabilities {
            typical_dimensions: Some(self.dim),
            ..Default::default()
        }
    }

    async fn embed(&self, text: &str) -> Result<Embedding, EmbeddingError> {
        Ok(Embedding::new(self.deterministic_vector(text)))
    }
}

/// LLM mock : titre fixe, contenu = prompt utilisateur.
pub struct InMemoryLlmProvider;

#[async_trait]
impl LlmProvider for InMemoryLlmProvider {
    fn name(&self) -> &'static str {
        "in-memory-llm"
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
        user: &str,
    ) -> Result<MemoryDraft, LlmError> {
        let title = user
            .split_whitespace()
            .take(4)
            .collect::<Vec<_>>()
            .join(" ");
        let title = if title.is_empty() {
            "Sans titre".into()
        } else {
            title
        };
        Ok(MemoryDraft::new(title, user))
    }

    async fn chat(&self, messages: &[crate::llm::ChatMessage]) -> Result<String, LlmError> {
        let user = messages
            .last()
            .map(|m| m.content.as_str())
            .unwrap_or_default();
        if let Some((_, body)) = user.rsplit_once("## Texte à analyser\n") {
            let draft = self.generate_memory_draft("", body).await?;
            return serde_json::to_string(&draft).map_err(|e| LlmError::StructuredOutputInvalid {
                provider: self.name().into(),
                message: e.to_string(),
            });
        }
        // Préprocesseur expand : renvoie le message original (simule un enrichissement neutre).
        if let Some(rest) = user.strip_prefix("## Message original\n") {
            if let Some((original, _)) = rest.split_once("\n\n## Ancrages Cortex\n") {
                return Ok(original.to_string());
            }
        }
        // Préprocesseur compress (map) : renvoie le segment tel quel.
        if let Some(rest) = user.strip_prefix("## Segment ") {
            if let Some((chunk, _)) = rest.split_once("\n\n") {
                return Ok(chunk.to_string());
            }
        }
        Ok(user.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex::Memory;

    #[tokio::test]
    async fn memory_repo_roundtrip() {
        let repo = InMemoryMemoryRepository::new();
        let mem = Memory::new("T", "C").unwrap();
        let id = mem.id;
        repo.save(&mem).await.unwrap();
        let loaded = repo.get_by_id(id).await.unwrap();
        assert_eq!(loaded.title, "T");
        assert_eq!(repo.list().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn vector_store_semantic_search_orders_by_score() {
        let store = InMemoryVectorStore::new();
        let id_a = MemoryId::new();
        let id_b = MemoryId::new();
        let query = vec![1.0, 0.0];
        store.upsert(id_a, &[0.9, 0.1]).await.unwrap();
        store.upsert(id_b, &[0.1, 0.9]).await.unwrap();

        let hits = store.semantic_search(&query, 2).await.unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].memory_id, id_a);
        assert!(hits[0].score > hits[1].score);
    }

    #[tokio::test]
    async fn embedding_is_deterministic() {
        let provider = InMemoryEmbeddingProvider::new(4);
        let a = provider.embed("hello").await.unwrap();
        let b = provider.embed("hello").await.unwrap();
        assert_eq!(a, b);
        let c = provider.embed("world").await.unwrap();
        assert_ne!(a, c);
    }

    #[tokio::test]
    async fn get_embedding_returns_cached_vector() {
        let store = InMemoryVectorStore::new();
        let id = MemoryId::new();
        let vec = vec![0.1, 0.9];
        store.upsert(id, &vec).await.unwrap();
        assert_eq!(store.get_embedding(id).await.unwrap(), Some(vec));
    }

    #[tokio::test]
    async fn mock_bundle_into_deps() {
        let bundle = MockBundle::new();
        let deps = bundle.into_deps();
        assert_eq!(deps.config.embedding_dim, 768);
    }
}
