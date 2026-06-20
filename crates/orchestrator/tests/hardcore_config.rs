//! Problème 6 — Configuration, sécurité et extensibilité

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::env;

use infrastructure::{build_embedding_provider, build_llm_provider};
use orchestrator::OrchestratorConfig;
use reqwest::Client;

#[tokio::test]
async fn intensity1_missing_xai_key_fails_build_without_panic() {
    let mut cfg = OrchestratorConfig::default();
    cfg.providers.primary_llm = "xai".into();
    cfg.providers.fallback_llm.clear();
    cfg.xai.api_key_env = "HARDCORE_MISSING_XAI_KEY_TEST".into();

    env::remove_var("HARDCORE_MISSING_XAI_KEY_TEST");

    let client = Client::builder().build().expect("client http");
    let err = match build_llm_provider(&cfg, &client) {
        Err(e) => e,
        Ok(_) => panic!("clé absente = erreur claire"),
    };
    let msg = err.to_string();
    assert!(msg.contains("HARDCORE_MISSING_XAI_KEY_TEST"));
    assert!(!msg.contains("sk-"), "aucun secret ne doit fuiter dans l'erreur");
}

#[tokio::test]
async fn intensity1_ollama_embedding_factory_builds_without_api_key() {
    let mut cfg = OrchestratorConfig::default();
    cfg.providers.primary_embedding = "ollama".into();
    let client = Client::builder().build().expect("client http");
    let provider = build_embedding_provider(&cfg, &client).expect("ollama embedding local");
    assert_eq!(provider.name(), "ollama");
}

#[test]
fn intensity1_incomplete_toml_uses_defaults() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("orchestrator.toml"), "[workspace]\npath = \"./m\"\n")
        .expect("write toml");
    let cfg = OrchestratorConfig::load_workspace(dir.path()).expect("charge defaults");
    assert_eq!(cfg.providers.primary_llm, "xai");
    assert_eq!(cfg.embedding_dim, 768);
}

#[test]
fn intensity2_provider_routing_uses_factory_not_use_cases() {
    let mut cfg = OrchestratorConfig::default();
    cfg.providers.primary_llm = "ollama".into();
    cfg.providers.fallback_llm.clear();
    let client = Client::builder().build().expect("client http");
    let provider = build_llm_provider(&cfg, &client).expect("ollama sans clé xAI");
    assert_eq!(provider.name(), "ollama");
}