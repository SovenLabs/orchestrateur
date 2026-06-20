use std::env;
use std::sync::Arc;

use orchestrator::{LlmProvider, OrchestratorConfig};
use reqwest::Client;
use thiserror::Error;
use tracing::warn;

use crate::providers::UnavailableLlmProvider;

use super::{ChainedLlmProvider, OllamaLlmProvider, XaiGrokProvider};

/// Erreurs de construction des providers LLM.
#[derive(Debug, Error)]
pub enum LlmFactoryError {
    /// Provider inconnu ou clé API manquante.
    #[error("llm factory: {0}")]
    Build(String),
}

/// Construit un [`LlmProvider`] avec fallbacks selon la config.
///
/// # Errors
///
/// Retourne [`LlmFactoryError`] si la résolution échoue.
pub fn build_llm_provider(
    config: &OrchestratorConfig,
    client: &Client,
) -> Result<Arc<dyn LlmProvider>, LlmFactoryError> {
    let mut names = vec![config.providers.primary_llm.clone()];
    names.extend(config.providers.fallback_llm.clone());

    let mut providers: Vec<Arc<dyn LlmProvider>> = Vec::new();
    let mut failures: Vec<String> = Vec::new();
    for name in names {
        if providers.iter().any(|p| p.name() == name) {
            continue;
        }
        match resolve_llm(name.as_str(), config, client) {
            Ok(provider) => providers.push(provider),
            Err(err) => {
                warn!(provider = %name, error = %err, "LLM provider ignoré au démarrage");
                failures.push(format!("{name}: {err}"));
            }
        }
    }

    if providers.is_empty() {
        let reason = if failures.is_empty() {
            "aucun LLM provider configuré".to_string()
        } else {
            failures.join("; ")
        };
        warn!(reason = %reason, "démarrage avec LLM indisponible (mode dégradé)");
        return Ok(Arc::new(UnavailableLlmProvider::new(reason)));
    }

    if providers.len() == 1 {
        Ok(providers.remove(0))
    } else {
        Ok(Arc::new(ChainedLlmProvider::new(providers)))
    }
}

fn resolve_llm(
    name: &str,
    config: &OrchestratorConfig,
    client: &Client,
) -> Result<Arc<dyn LlmProvider>, LlmFactoryError> {
    match name {
        "xai" => {
            let key = env::var(&config.xai.api_key_env).map_err(|_| {
                LlmFactoryError::Build(format!(
                    "variable d'environnement {} introuvable",
                    config.xai.api_key_env
                ))
            })?;
            Ok(Arc::new(XaiGrokProvider::new(
                client.clone(),
                key,
                config.xai.model.clone(),
                config.xai.timeout_secs,
                config.xai.max_retries,
            )))
        }
        "ollama" => Ok(Arc::new(OllamaLlmProvider::new(
            client.clone(),
            config.ollama.url.clone(),
            config.ollama.chat_model.clone(),
            config.ollama.timeout_secs,
            config.ollama.max_retries,
        ))),
        other => Err(LlmFactoryError::Build(format!(
            "LLM provider inconnu: {other}"
        ))),
    }
}
