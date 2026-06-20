use std::sync::Arc;

use cortex::{EmbeddingProvider, MemoryRepository, VectorStore};

use crate::config::OrchestratorConfig;
use crate::events::{EventPublisher, TracingEventPublisher};
use crate::llm::LlmProvider;
use crate::security::{build_test_security_context, SecurityContext};

/// Injection des ports Cortex — seul point de couplage vers l'infrastructure future.
#[derive(Clone)]
pub struct AppDependencies {
    /// Persistance des mémoires Markdown.
    pub memory_repo: Arc<dyn MemoryRepository>,
    /// Index vectoriel local.
    pub vector_store: Arc<dyn VectorStore>,
    /// Génération d'embeddings.
    pub embedding: Arc<dyn EmbeddingProvider>,
    /// Génération LLM (Structured Outputs, chat).
    pub llm: Arc<dyn LlmProvider>,
    /// Configuration applicative.
    pub config: OrchestratorConfig,
    /// Publication des événements de domaine.
    pub events: Arc<dyn EventPublisher>,
    /// Contexte de sécurité (couches 2–4).
    pub security: Arc<SecurityContext>,
}

impl AppDependencies {
    /// Construit les dépendances avec publisher [`TracingEventPublisher`].
    #[must_use]
    pub fn new(
        memory_repo: Arc<dyn MemoryRepository>,
        vector_store: Arc<dyn VectorStore>,
        embedding: Arc<dyn EmbeddingProvider>,
        llm: Arc<dyn LlmProvider>,
        config: OrchestratorConfig,
        security: Arc<SecurityContext>,
    ) -> Self {
        Self::with_events(
            memory_repo,
            vector_store,
            embedding,
            llm,
            config,
            Arc::new(TracingEventPublisher),
            security,
        )
    }

    /// Construit les dépendances avec un publisher personnalisé.
    #[must_use]
    pub fn with_events(
        memory_repo: Arc<dyn MemoryRepository>,
        vector_store: Arc<dyn VectorStore>,
        embedding: Arc<dyn EmbeddingProvider>,
        llm: Arc<dyn LlmProvider>,
        config: OrchestratorConfig,
        events: Arc<dyn EventPublisher>,
        security: Arc<SecurityContext>,
    ) -> Self {
        Self {
            memory_repo,
            vector_store,
            embedding,
            llm,
            config,
            events,
            security,
        }
    }

    /// Construit des dépendances de test avec sécurité relâchée.
    #[must_use]
    pub fn for_tests(
        memory_repo: Arc<dyn MemoryRepository>,
        vector_store: Arc<dyn VectorStore>,
        embedding: Arc<dyn EmbeddingProvider>,
        llm: Arc<dyn LlmProvider>,
        config: OrchestratorConfig,
        events: Arc<dyn EventPublisher>,
    ) -> Self {
        let security = build_test_security_context(&config);
        Self::with_events(
            memory_repo,
            vector_store,
            embedding,
            llm,
            config,
            events,
            security,
        )
    }
}
