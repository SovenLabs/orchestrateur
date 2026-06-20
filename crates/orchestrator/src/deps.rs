use std::sync::Arc;

use cortex::{EmbeddingProvider, MemoryRepository, VectorStore};

use crate::config::OrchestratorConfig;
use crate::events::{EventPublisher, TracingEventPublisher};

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
    /// Publication des événements de domaine.
    pub events: Arc<dyn EventPublisher>,
}

impl AppDependencies {
    /// Construit les dépendances avec publisher [`TracingEventPublisher`].
    #[must_use]
    pub fn new(
        memory_repo: Arc<dyn MemoryRepository>,
        vector_store: Arc<dyn VectorStore>,
        embedding: Arc<dyn EmbeddingProvider>,
        config: OrchestratorConfig,
    ) -> Self {
        Self::with_events(
            memory_repo,
            vector_store,
            embedding,
            config,
            Arc::new(TracingEventPublisher),
        )
    }

    /// Construit les dépendances avec un publisher personnalisé.
    #[must_use]
    pub fn with_events(
        memory_repo: Arc<dyn MemoryRepository>,
        vector_store: Arc<dyn VectorStore>,
        embedding: Arc<dyn EmbeddingProvider>,
        config: OrchestratorConfig,
        events: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            memory_repo,
            vector_store,
            embedding,
            config,
            events,
        }
    }
}