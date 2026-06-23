use std::sync::Arc;

use b212::{B212Journal, MarketDataProvider, ProposalRepository, SimTradeRepository};
use cortex::{EmbeddingProvider, MemoryRepository, SessionRepository, VectorStore};

use crate::config::OrchestratorConfig;
use crate::draft::DraftRepository;
use crate::events::{EventPublisher, TracingEventPublisher};
use crate::llm::LlmProvider;
use crate::mcp::McpGateway;
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
    /// Sessions de conversation agent (Phase 7).
    pub session_repo: Arc<dyn SessionRepository>,
    /// File de brouillons insight en attente de revue.
    pub draft_repo: Arc<dyn DraftRepository>,
    /// Configuration applicative.
    pub config: OrchestratorConfig,
    /// Publication des événements de domaine.
    pub events: Arc<dyn EventPublisher>,
    /// Contexte de sécurité (couches 2–4).
    pub security: Arc<SecurityContext>,
    /// Gateway MCP optionnel (Phase 9).
    pub mcp: Option<Arc<dyn McpGateway>>,
    /// Données marché B212 (fixtures Phase 3, adapters live ultérieurs).
    pub market_data: Option<Arc<dyn MarketDataProvider>>,
    /// Journal audit B212 JSONL.
    pub b212_journal: Option<Arc<dyn B212Journal>>,
    /// Propositions trade HITL B212.
    pub b212_proposals: Option<Arc<dyn ProposalRepository>>,
    /// Fills paper B212.
    pub b212_sim_trades: Option<Arc<dyn SimTradeRepository>>,
}

impl AppDependencies {
    /// Construit les dépendances avec publisher [`TracingEventPublisher`].
    #[must_use]
    pub fn new(
        memory_repo: Arc<dyn MemoryRepository>,
        vector_store: Arc<dyn VectorStore>,
        embedding: Arc<dyn EmbeddingProvider>,
        llm: Arc<dyn LlmProvider>,
        session_repo: Arc<dyn SessionRepository>,
        draft_repo: Arc<dyn DraftRepository>,
        config: OrchestratorConfig,
        security: Arc<SecurityContext>,
        mcp: Option<Arc<dyn McpGateway>>,
        market_data: Option<Arc<dyn MarketDataProvider>>,
        b212_journal: Option<Arc<dyn B212Journal>>,
        b212_proposals: Option<Arc<dyn ProposalRepository>>,
        b212_sim_trades: Option<Arc<dyn SimTradeRepository>>,
    ) -> Self {
        Self::with_events(
            memory_repo,
            vector_store,
            embedding,
            llm,
            session_repo,
            draft_repo,
            config,
            Arc::new(TracingEventPublisher),
            security,
            mcp,
            market_data,
            b212_journal,
            b212_proposals,
            b212_sim_trades,
        )
    }

    /// Construit les dépendances avec un publisher personnalisé.
    #[must_use]
    pub fn with_events(
        memory_repo: Arc<dyn MemoryRepository>,
        vector_store: Arc<dyn VectorStore>,
        embedding: Arc<dyn EmbeddingProvider>,
        llm: Arc<dyn LlmProvider>,
        session_repo: Arc<dyn SessionRepository>,
        draft_repo: Arc<dyn DraftRepository>,
        config: OrchestratorConfig,
        events: Arc<dyn EventPublisher>,
        security: Arc<SecurityContext>,
        mcp: Option<Arc<dyn McpGateway>>,
        market_data: Option<Arc<dyn MarketDataProvider>>,
        b212_journal: Option<Arc<dyn B212Journal>>,
        b212_proposals: Option<Arc<dyn ProposalRepository>>,
        b212_sim_trades: Option<Arc<dyn SimTradeRepository>>,
    ) -> Self {
        Self {
            memory_repo,
            vector_store,
            embedding,
            llm,
            session_repo,
            draft_repo,
            config,
            events,
            security,
            mcp,
            market_data,
            b212_journal,
            b212_proposals,
            b212_sim_trades,
        }
    }

    /// Construit des dépendances de test avec sécurité relâchée.
    #[must_use]
    pub fn for_tests(
        memory_repo: Arc<dyn MemoryRepository>,
        vector_store: Arc<dyn VectorStore>,
        embedding: Arc<dyn EmbeddingProvider>,
        llm: Arc<dyn LlmProvider>,
        session_repo: Arc<dyn SessionRepository>,
        draft_repo: Arc<dyn DraftRepository>,
        config: OrchestratorConfig,
        events: Arc<dyn EventPublisher>,
    ) -> Self {
        let security = build_test_security_context(&config);
        Self::with_events(
            memory_repo,
            vector_store,
            embedding,
            llm,
            session_repo,
            draft_repo,
            config,
            events,
            security,
            None,
            None,
            None,
            None,
            None,
        )
    }
}
