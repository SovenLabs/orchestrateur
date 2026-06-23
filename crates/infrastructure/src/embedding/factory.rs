use std::sync::Arc;

use cortex::EmbeddingProvider;
use orchestrator::OrchestratorConfig;
use reqwest::Client;
use thiserror::Error;
use tracing::warn;

use crate::providers::{resolve_embedding_from_registry, UnavailableEmbeddingProvider};

use super::{CachedEmbeddingProvider, ChainedEmbeddingProvider};

/// Taille par défaut du cache LRU d'embeddings.
const DEFAULT_EMBEDDING_CACHE_ENTRIES: usize = 4096;

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
    let mut failures: Vec<String> = Vec::new();
    for name in names {
        if providers.iter().any(|p| p.name() == name) {
            continue;
        }
        match resolve_embedding_from_registry(name.as_str(), config, client) {
            Ok(provider) => providers.push(provider),
            Err(err) => {
                warn!(provider = %name, error = %err, "embedding provider ignoré au démarrage");
                failures.push(format!("{name}: {err}"));
            }
        }
    }

    if providers.is_empty() {
        let reason = if failures.is_empty() {
            "aucun embedding provider configuré".to_string()
        } else {
            failures.join("; ")
        };
        warn!(reason = %reason, "démarrage avec embeddings indisponibles (mode dégradé)");
        return Ok(Arc::new(UnavailableEmbeddingProvider::new(reason)));
    }

    let provider: Arc<dyn EmbeddingProvider> = if providers.len() == 1 {
        providers.remove(0)
    } else {
        Arc::new(ChainedEmbeddingProvider::new(providers))
    };

    if provider.name() == "unavailable" {
        return Ok(provider);
    }

    Ok(CachedEmbeddingProvider::new(
        provider,
        DEFAULT_EMBEDDING_CACHE_ENTRIES,
    ))
}
