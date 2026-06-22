//! Problème 5 — Gestion des erreurs et observabilité

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use orchestrator::testing::{
    build_test_facade, CollectingEventPublisher, FailNthVectorStore, InvalidJsonLlmProvider,
    MockBundle,
};
use orchestrator::{AppDependencies, OrchestratorError};

#[tokio::test]
#[ignore = "intégration: erreur structurée JSON LLM invalide"]
async fn intensity1_invalid_json_returns_structured_error() {
    let bundle = MockBundle::new();
    let events = CollectingEventPublisher::new();
    let llm: Arc<dyn orchestrator::LlmProvider> = Arc::new(InvalidJsonLlmProvider);
    let deps = AppDependencies::for_tests(
        bundle.memory_repo,
        bundle.vector_store,
        bundle.embedding,
        llm,
        bundle.session_repo,
        bundle.config,
        events.clone(),
    );

    let err = build_test_facade(deps)
        .assimilate("json cassé", None)
        .await
        .expect_err("JSON invalide doit échouer proprement");

    assert!(matches!(
        err,
        OrchestratorError::Llm(orchestrator::LlmError::StructuredOutputInvalid { .. })
    ));
    assert!(
        events.domain_events().is_empty(),
        "pas d'événement domaine si assimilation échoue avant persistance"
    );
}

#[tokio::test]
#[ignore = "intégration: événements usage LLM"]
async fn intensity1_llm_usage_emitted_on_success() {
    let bundle = MockBundle::new();
    let events = CollectingEventPublisher::new();
    let deps = AppDependencies::for_tests(
        bundle.memory_repo,
        bundle.vector_store,
        bundle.embedding,
        bundle.llm,
        bundle.session_repo,
        bundle.config,
        events.clone(),
    );

    build_test_facade(deps)
        .assimilate("trace usage tokens", None)
        .await
        .expect("assimilation mock");

    assert!(
        !events.domain_events().is_empty(),
        "événements domaine émis"
    );
}

#[tokio::test]
#[ignore = "intégration: échec vector store remonté"]
async fn intensity2_vector_store_failure_surfaces_as_cortex_error() {
    let bundle = MockBundle::new();
    let failing_store = FailNthVectorStore::new(bundle.vector_store.clone(), 1);
    let deps = AppDependencies::for_tests(
        bundle.memory_repo,
        failing_store,
        bundle.embedding,
        bundle.llm,
        bundle.session_repo,
        bundle.config,
        Arc::new(orchestrator::NoopEventPublisher),
    );

    let err = build_test_facade(deps)
        .assimilate("échec vector store", None)
        .await
        .expect_err("upsert simulé doit échouer");

    assert!(matches!(
        err,
        OrchestratorError::Cortex(cortex::CortexError::GraphError(_))
    ));
}
