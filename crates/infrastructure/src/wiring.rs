use std::sync::Arc;

use cortex::{EmbeddingProvider, MemoryRepository, VectorStore};
use orchestrator::{
    build_security_context, AppDependencies, LlmProvider, OrchestratorConfig, SecurityBootstrapError,
};
use reqwest::Client;
use thiserror::Error;

use crate::embedding::{build_embedding_provider, EmbeddingFactoryError};
use crate::llm::{build_llm_provider, LlmFactoryError};
use crate::memory_repository::FileMemoryRepository;
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

    Ok(AppDependencies::new(
        memory_repo,
        vector_store,
        embedding,
        llm,
        config,
        security,
    ))
}