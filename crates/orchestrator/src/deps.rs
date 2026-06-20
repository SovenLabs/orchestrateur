use std::sync::Arc;

use cortex::{EmbeddingProvider, MemoryRepository, VectorStore};

use crate::config::OrchestratorConfig;

/// Injection des ports Cortex — seul point de couplage vers l'infrastructure future.
#[derive(Clone)]
pub struct AppDependencies {
    /// Persistance des mémoires Markdown.
    pub memory_repo: Arc<dyn MemoryRepository>,
    /// Index vectoriel local.
    pub vector_store: Arc<dyn VectorStore>,
    /// Génération d'embeddings.
    pub embedding: Arc<dyn EmbeddingProvider>,
    /// Configuration applicative.
    pub config: OrchestratorConfig,
}

impl AppDependencies {
    /// Construit les dépendances à partir des ports injectés.
    pub fn new(
        memory_repo: Arc<dyn MemoryRepository>,
        vector_store: Arc<dyn VectorStore>,
        embedding: Arc<dyn EmbeddingProvider>,
        config: OrchestratorConfig,
    ) -> Self {
        Self {
            memory_repo,
            vector_store,
            embedding,
            config,
        }
    }
}