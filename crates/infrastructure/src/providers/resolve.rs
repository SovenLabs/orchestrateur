use std::env;
use std::sync::Arc;

use cortex::EmbeddingProvider;
use orchestrator::{LlmProvider, OrchestratorConfig, ProviderProfile, ProviderRegistry};
use reqwest::Client;

use crate::embedding::{EmbeddingFactoryError, OllamaEmbeddingProvider, OpenAiEmbeddingsProvider};
use crate::llm::{
    AnthropicLlmProvider, LlmFactoryError, OllamaLlmProvider, OpenAiCompatibleLlmProvider,
    XaiGrokProvider,
};

/// Résout un provider LLM via le registre typé Phase 9.
pub fn resolve_llm_from_registry(
    name: &str,
    config: &OrchestratorConfig,
    client: &Client,
) -> Result<Arc<dyn LlmProvider>, LlmFactoryError> {
    let registry = ProviderRegistry::new();
    let descriptor = registry
        .llm_descriptor(name)
        .ok_or_else(|| LlmFactoryError::Build(format!("LLM provider inconnu: {name}")))?;
    let profile = config.provider_profiles.resolve(name, descriptor);
    if !profile.enabled {
        return Err(LlmFactoryError::Build(format!("provider {name} désactivé")));
    }
    build_llm_from_profile(descriptor.id, &profile, config, client)
}

/// Résout un provider embedding via le registre typé Phase 9.
pub fn resolve_embedding_from_registry(
    name: &str,
    config: &OrchestratorConfig,
    client: &Client,
) -> Result<Arc<dyn EmbeddingProvider>, EmbeddingFactoryError> {
    let registry = ProviderRegistry::new();
    let descriptor = registry.embedding_descriptor(name).ok_or_else(|| {
        EmbeddingFactoryError::Build(format!("embedding provider inconnu: {name}"))
    })?;
    let profile = config.provider_profiles.resolve(name, descriptor);
    if !profile.enabled {
        return Err(EmbeddingFactoryError::Build(format!(
            "provider {name} désactivé"
        )));
    }
    build_embedding_from_profile(descriptor.id, &profile, config, client)
}

fn build_llm_from_profile(
    id: &'static str,
    profile: &ProviderProfile,
    config: &OrchestratorConfig,
    client: &Client,
) -> Result<Arc<dyn LlmProvider>, LlmFactoryError> {
    match id {
        "xai" => {
            let key = resolve_api_key(profile).map_err(LlmFactoryError::Build)?;
            Ok(Arc::new(XaiGrokProvider::new(
                client.clone(),
                key,
                pick_model(profile, &config.xai.model),
                profile.timeout_secs,
                profile.max_retries,
            )))
        }
        "ollama" => Ok(Arc::new(OllamaLlmProvider::new(
            client.clone(),
            pick_base_url(profile, &config.ollama.url),
            pick_model(profile, &config.ollama.chat_model),
            profile.timeout_secs,
            profile.max_retries,
        ))),
        "anthropic" => {
            let key = resolve_api_key(profile).map_err(LlmFactoryError::Build)?;
            Ok(Arc::new(AnthropicLlmProvider::new(
                client.clone(),
                key,
                profile.model.clone(),
                profile.base_url.clone(),
                profile.timeout_secs,
                profile.max_retries,
            )))
        }
        "openai" | "groq" | "openrouter" | "together" | "deepseek" | "mistral" | "perplexity"
        | "lmstudio" | "azure_openai" => {
            let key = resolve_api_key_optional(profile).map_err(LlmFactoryError::Build)?;
            Ok(Arc::new(OpenAiCompatibleLlmProvider::new(
                id,
                client.clone(),
                key,
                profile.model.clone(),
                profile.base_url.clone(),
                profile.timeout_secs,
                profile.max_retries,
            )))
        }
        other => Err(LlmFactoryError::Build(format!(
            "résolution LLM non implémentée: {other}"
        ))),
    }
}

fn build_embedding_from_profile(
    id: &'static str,
    profile: &ProviderProfile,
    config: &OrchestratorConfig,
    client: &Client,
) -> Result<Arc<dyn EmbeddingProvider>, EmbeddingFactoryError> {
    let expected_dim = Some(config.vector_store.embedding_dimension);
    match id {
        "ollama" | "fastembed" => Ok(Arc::new(OllamaEmbeddingProvider::new(
            client.clone(),
            pick_base_url(profile, &config.ollama.url),
            pick_model(profile, &config.ollama.embedding_model),
            profile.timeout_secs,
            profile.max_retries,
            expected_dim,
        ))),
        "openai" | "voyage" | "huggingface" => {
            let key = resolve_api_key(profile).map_err(EmbeddingFactoryError::Build)?;
            Ok(Arc::new(OpenAiEmbeddingsProvider::new(
                id,
                client.clone(),
                key,
                profile.model.clone(),
                profile.base_url.clone(),
                profile.timeout_secs,
                profile.max_retries,
                expected_dim,
            )))
        }
        other => Err(EmbeddingFactoryError::Build(format!(
            "résolution embedding non implémentée: {other}"
        ))),
    }
}

fn resolve_api_key(profile: &ProviderProfile) -> Result<String, String> {
    if profile.api_key_env.is_empty() {
        return Err("clé API requise mais api_key_env vide".into());
    }
    env::var(&profile.api_key_env).map_err(|_| {
        format!(
            "variable d'environnement {} introuvable",
            profile.api_key_env
        )
    })
}

fn resolve_api_key_optional(profile: &ProviderProfile) -> Result<String, String> {
    if profile.api_key_env.is_empty() {
        return Ok(String::new());
    }
    resolve_api_key(profile)
}

fn pick_model(profile: &ProviderProfile, fallback: &str) -> String {
    if profile.model.is_empty() {
        fallback.to_string()
    } else {
        profile.model.clone()
    }
}

fn pick_base_url(profile: &ProviderProfile, fallback: &str) -> String {
    if profile.base_url.is_empty() {
        fallback.to_string()
    } else {
        profile.base_url.clone()
    }
}