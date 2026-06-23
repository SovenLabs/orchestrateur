use std::sync::Arc;

use cortex::{EmbeddingProvider, MemoryRepository, SessionRepository, VectorStore};
use mcp::build_mcp_gateway;
use orchestrator::mcp::McpGateway;
use orchestrator::{
    build_security_context, AppDependencies, LlmProvider, OrchestratorConfig,
    SecurityBootstrapError,
};
use tracing::warn;
use reqwest::Client;
use thiserror::Error;

use b212::MarketDataProvider;
use crate::b212::FixtureMarketDataProvider;
use crate::embedding::{build_embedding_provider, EmbeddingFactoryError};
use crate::llm::{build_llm_provider, LlmFactoryError};
use crate::draft_repository::FileDraftRepository;
use crate::memory_repository::FileMemoryRepository;
use crate::session_store::SqliteSessionStore;
use crate::vector_store::{build_vector_store, VectorStoreFactoryError};

/// Erreurs de composition des dépendances applicatives.
#[derive(Debug, Error)]
pub enum WiringError {
    /// Factory vector store.
    #[error(transparent)]
    VectorStore(#[from] VectorStoreFactoryError),
    /// Factory embeddings.
    #[error(transparent)]
    Embedding(#[from] EmbeddingFactoryError),
    /// Factory LLM.
    #[error(transparent)]
    Llm(#[from] LlmFactoryError),
    /// Mode mémoire (tests) demandé hors contexte test.
    #[error("vector store type=memory : utiliser MockBundle en tests")]
    MemoryMode,
    /// Initialisation sécurité.
    #[error(transparent)]
    Security(#[from] SecurityBootstrapError),
}

/// Construit [`AppDependencies`] complètes pour la production (FS + `LanceDB` + providers).
///
/// Pour les tests unitaires, préférer [`orchestrator::testing::MockBundle`].
///
/// # Errors
///
/// Retourne [`WiringError`] si un adapter ne peut pas être instancié.
pub async fn build_app_dependencies(
    config: OrchestratorConfig,
) -> Result<AppDependencies, WiringError> {
    if config.vector_store.store_type == "memory" {
        return Err(WiringError::MemoryMode);
    }

    std::fs::create_dir_all(config.memories_dir())
        .map_err(|e| WiringError::VectorStore(VectorStoreFactoryError::Build(e.to_string())))?;

    let client = Client::builder()
        .build()
        .map_err(|e| WiringError::Embedding(EmbeddingFactoryError::Build(e.to_string())))?;

    let security = build_security_context(&config)?;

    let memory_repo: Arc<dyn MemoryRepository> =
        Arc::new(FileMemoryRepository::new(config.memories_dir()));
    security
        .seed_honeypots_if_needed(memory_repo.as_ref(), &config)
        .await
        .map_err(|e| WiringError::VectorStore(VectorStoreFactoryError::Build(e.to_string())))?;

    let vector_store: Arc<dyn VectorStore> = build_vector_store(&config).await?;
    let embedding: Arc<dyn EmbeddingProvider> = build_embedding_provider(&config, &client)?;
    let llm: Arc<dyn LlmProvider> = build_llm_provider(&config, &client)?;

    let mcp: Option<std::sync::Arc<dyn McpGateway>> = if config.mcp.enabled {
        match build_mcp_gateway(&config.mcp).await {
            Ok(gateway) => gateway,
            Err(err) => {
                warn!(%err, "MCP désactivé — aucun serveur n'a démarré");
                None
            }
        }
    } else {
        None
    };

    let session_repo: Arc<dyn SessionRepository> = Arc::new(
        SqliteSessionStore::open(config.sessions_db_path())
            .map_err(|e| WiringError::VectorStore(VectorStoreFactoryError::Build(e.to_string())))?,
    );

    let draft_repo: Arc<dyn orchestrator::draft::DraftRepository> =
        Arc::new(FileDraftRepository::new(config.drafts_dir()));

    let market_data: Option<Arc<dyn MarketDataProvider>> = if config.b212.enabled {
        let fixtures_dir = config.b212_fixtures_dir();
        std::fs::create_dir_all(&fixtures_dir).map_err(|e| {
            WiringError::VectorStore(VectorStoreFactoryError::Build(format!(
                "b212 fixtures {}: {e}",
                fixtures_dir.display()
            )))
        })?;
        std::fs::create_dir_all(config.b212_journal_dir()).map_err(|e| {
            WiringError::VectorStore(VectorStoreFactoryError::Build(format!(
                "b212 journal: {e}"
            )))
        })?;
        std::fs::create_dir_all(config.b212_proposals_dir()).map_err(|e| {
            WiringError::VectorStore(VectorStoreFactoryError::Build(format!(
                "b212 proposals: {e}"
            )))
        })?;
        Some(Arc::new(FixtureMarketDataProvider::new(fixtures_dir)))
    } else {
        None
    };

    Ok(AppDependencies::new(
        memory_repo,
        vector_store,
        embedding,
        llm,
        session_repo,
        draft_repo,
        config,
        security,
        mcp,
        market_data,
    ))
}
