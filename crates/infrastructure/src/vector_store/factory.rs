use std::sync::Arc;

use cortex::VectorStore;
use orchestrator::OrchestratorConfig;
use thiserror::Error;

use super::LancedbVectorStore;

/// Erreurs de construction du vector store.
#[derive(Debug, Error)]
pub enum VectorStoreFactoryError {
    /// Erreur `LanceDB` ou I/O.
    #[error("vector store: {0}")]
    Build(String),
}

/// Construit un [`VectorStore`] selon la configuration TOML.
///
/// # Errors
///
/// Retourne [`VectorStoreFactoryError`] si le type est inconnu ou `LanceDB` échoue.
pub async fn build_vector_store(
    config: &OrchestratorConfig,
) -> Result<Arc<dyn VectorStore>, VectorStoreFactoryError> {
    match config.vector_store.store_type.as_str() {
        "lancedb" => {
            let store = LancedbVectorStore::open(
                config.lancedb_path(),
                config.vector_store.embedding_dimension,
            )
            .await
            .map_err(|e| VectorStoreFactoryError::Build(e.to_string()))?;
            Ok(Arc::new(store))
        }
        other => Err(VectorStoreFactoryError::Build(format!(
            "type vector store non supporté en infrastructure: {other} (utiliser memory via mocks ou lancedb)"
        ))),
    }
}