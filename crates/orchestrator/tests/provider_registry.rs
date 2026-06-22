//! Tests registre providers Phase 9.

use orchestrator::{ProviderKind, ProviderRegistry};

#[test]
fn registry_lists_twelve_llm_providers() {
    let registry = ProviderRegistry::new();
    assert_eq!(registry.llm_descriptors().len(), 12);
}

#[test]
fn registry_lists_five_embedding_providers() {
    let registry = ProviderRegistry::new();
    assert_eq!(registry.embedding_descriptors().len(), 5);
}

#[test]
fn registry_total_at_least_fifteen() {
    let registry = ProviderRegistry::new();
    assert!(registry.total_count() >= 15);
}

#[test]
fn openrouter_is_openai_compatible() {
    let registry = ProviderRegistry::new();
    let d = registry.llm_descriptor("openrouter").expect("openrouter");
    assert_eq!(d.kind, ProviderKind::Llm);
    assert_eq!(d.default_api_key_env, "OPENROUTER_API_KEY");
}