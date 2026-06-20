use std::sync::Arc;

use cortex::EmbeddingProvider;
use orchestrator::OrchestratorConfig;
use reqwest::Client;
use thiserror::Error;

use super::{ChainedEmbeddingProvider, OllamaEmbeddingProvider};

/// Erreurs de construction des providers d'embeddings.
#[derive(Debug, Error)]
pub enum EmbeddingFactoryError {
    /// Provider inconnu ou configuration invalide.
    #[error("embedding factory: {0}")]
    Build(String),
}

/// Construit un [`EmbeddingProvider`] avec fallbacks selon la config.
///
/// # Errors
///
/// Retourne [`EmbeddingFactoryError`] si aucun provider n'est résolu.
pub fn build_embedding_provider(
    config: &OrchestratorConfig,
    client: &Client,
) -> Result<Arc<dyn EmbeddingProvider>, EmbeddingFactoryError> {
    let mut names = vec![config.providers.primary_embedding.clone()];
    names.extend(config.providers.fallback_embedding.clone());

    let mut providers: Vec<Arc<dyn EmbeddingProvider>> = Vec::new();
    for name in names {
        if providers.iter().any(|p| p.name() == name) {
            continue;
        }
        let provider = resolve_embedding(name.as_str(), config, client)?;
        providers.push(provider);
    }

    if providers.is_empty() {
        return Err(EmbeddingFactoryError::Build(
            "aucun embedding provider configuré".into(),
        ));
    }

    if providers.len() == 1 {
        Ok(providers.remove(0))
    } else {
        Ok(Arc::new(ChainedEmbeddingProvider::new(providers)))
    }
}

fn resolve_embedding(
    name: &str,
    config: &OrchestratorConfig,
    client: &Client,
) -> Result<Arc<dyn EmbeddingProvider>, EmbeddingFactoryError> {
    match name {
        "ollama" => Ok(Arc::new(OllamaEmbeddingProvider::new(
            client.clone(),
            config.ollama.url.clone(),
            config.ollama.embedding_model.clone(),
            config.ollama.timeout_secs,
            config.ollama.max_retries,
            Some(config.vector_store.embedding_dimension),
        ))),
        other => Err(EmbeddingFactoryError::Build(format!(
            "embedding provider inconnu: {other}"
        ))),
    }
}