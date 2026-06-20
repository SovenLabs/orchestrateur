use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use cortex::{
    cosine_similarity, CortexError, EmbeddingProvider, Memory, MemoryId, MemoryRepository,
    SearchFilter, SearchHit, VectorStore,
};

use crate::config::OrchestratorConfig;
use crate::deps::AppDependencies;

/// Bundle prêt à l'emploi : les trois mocks + configuration par défaut.
pub struct MockBundle {
    /// Mock de persistance mémoires.
    pub memory_repo: Arc<InMemoryMemoryRepository>,
    /// Mock de vector store.
    pub vector_store: Arc<InMemoryVectorStore>,
    /// Mock d'embeddings déterministes.
    pub embedding: Arc<InMemoryEmbeddingProvider>,
    /// Configuration de test.
    pub config: OrchestratorConfig,
}

impl MockBundle {
    /// Crée un bundle de mocks thread-safe avec configuration par défaut.
    pub fn new() -> Self {
        let config = OrchestratorConfig::default();
        let embedding = Arc::new(InMemoryEmbeddingProvider::new(config.embedding_dim));
        Self {
            memory_repo: Arc::new(InMemoryMemoryRepository::new()),
            vector_store: Arc::new(InMemoryVectorStore::new()),
            embedding,
            config,
        }
    }

    /// Convertit le bundle en [`AppDependencies`].
    pub fn into_deps(self) -> AppDependencies {
        AppDependencies::new(
            self.memory_repo,
            self.vector_store,
            self.embedding,
            self.config,
        )
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
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    /// Nombre de mémoires stockées (utilitaire de test).
    pub fn len(&self) -> usize {
        self.inner.read().map(|m| m.len()).unwrap_or(0)
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
        guard.get(&id).cloned().ok_or(CortexError::MemoryNotFound(id))
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
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    /// Nombre de vecteurs indexés (utilitaire de test).
    pub fn len(&self) -> usize {
        self.inner.read().map(|m| m.len()).unwrap_or(0)
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
        let limit = filter.limit.unwrap_or(10);
        let mut hits = self.semantic_search(query_embedding, limit).await?;

        if let Some(min) = filter.min_score {
            hits.retain(|h| h.score >= min);
        }

        if !filter.tags.is_empty() {
            // Le filtrage par tags est effectué dans le use case via MemoryRepository.
        }

        Ok(hits)
    }

    async fn delete(&self, memory_id: MemoryId) -> Result<(), CortexError> {
        let mut guard = self
            .inner
            .write()
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
        guard.remove(&memory_id).ok_or(CortexError::MemoryNotFound(memory_id))?;
        Ok(())
    }
}

/// Embeddings déterministes dérivés du hash du texte (reproductibles en test).
pub struct InMemoryEmbeddingProvider {
    dim: usize,
}

impl InMemoryEmbeddingProvider {
    /// Crée un provider avec la dimension indiquée.
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
    async fn embed(&self, text: &str) -> Result<Vec<f32>, CortexError> {
        Ok(self.deterministic_vector(text))
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, CortexError> {
        let mut out = Vec::with_capacity(texts.len());
        for text in texts {
            out.push(self.embed(text).await?);
        }
        Ok(out)
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
    async fn mock_bundle_into_deps() {
        let bundle = MockBundle::new();
        let deps = bundle.into_deps();
        assert_eq!(deps.config.embedding_dim, 8);
    }
}