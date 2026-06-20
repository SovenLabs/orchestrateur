//! Problème 2 — Résilience multi-provider (xAI ↔ Ollama)

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use infrastructure::ChainedLlmProvider;
use orchestrator::memory_draft::MemoryDraft;
use orchestrator::testing::{
    build_test_facade, CollectingEventPublisher, MockBundle, ScriptedLlmProvider,
    StableOllamaLlmProvider,
};
use orchestrator::{AppDependencies, OrchestratorFacade};

fn chained_facade(xai: Arc<dyn orchestrator::LlmProvider>) -> OrchestratorFacade {
    let bundle = MockBundle::new();
    let events = CollectingEventPublisher::new();
    let llm: Arc<dyn orchestrator::LlmProvider> = Arc::new(ChainedLlmProvider::new(vec![
        xai,
        Arc::new(StableOllamaLlmProvider),
    ]));
    let deps = AppDependencies::for_tests(
        bundle.memory_repo,
        bundle.vector_store,
        bundle.embedding,
        llm,
        bundle.config,
        events,
    );
    build_test_facade(deps)
}

#[tokio::test]
async fn intensity1_fallback_xai_429_then_ollama_success() {
    let draft = MemoryDraft {
        title: "Ne sera pas utilisé".into(),
        content: "x".into(),
        tags: vec![],
        backlinks: vec![],
    };
    let xai = ScriptedLlmProvider::xai_fail_429_then_ok(draft);
    let facade = chained_facade(xai.clone());

    let (memory, _) = facade
        .assimilate("test résilience fallback", None)
        .await
        .expect("le fallback Ollama doit réussir");

    assert_eq!(memory.title, "Ollama fallback");
    assert_eq!(
        xai.call_count(),
        1,
        "xAI ne doit être appelé qu'une fois avant fallback"
    );
}

#[tokio::test]
async fn intensity1_xai_auth_error_does_not_fallback() {
    let xai = ScriptedLlmProvider::new(
        "xai",
        vec![Err(orchestrator::LlmError::AuthenticationFailed {
            provider: "xai".into(),
        })],
    );
    let facade = chained_facade(xai);

    let err = facade
        .assimilate("auth failure", None)
        .await
        .expect_err("auth ne doit pas basculer vers Ollama");

    assert!(matches!(
        err,
        orchestrator::OrchestratorError::Llm(orchestrator::LlmError::AuthenticationFailed { .. })
    ));
}

#[tokio::test]
#[ignore = "hardcore intensity 2 — simulation longue indisponibilité xAI"]
async fn intensity2_xai_unavailable_then_recovery() {
    let mut script = Vec::new();
    for _ in 0..5 {
        script.push(Err(orchestrator::LlmError::Unavailable {
            provider: "xai".into(),
            message: "simulation panne".into(),
        }));
    }
    script.push(Ok(MemoryDraft {
        title: "Récupération xAI".into(),
        content: "ok".into(),
        tags: vec![],
        backlinks: vec![],
    }));

    let xai = ScriptedLlmProvider::new("xai", script);
    let ollama = Arc::new(StableOllamaLlmProvider);
    let bundle = MockBundle::new();
    let llm: Arc<dyn orchestrator::LlmProvider> =
        Arc::new(ChainedLlmProvider::new(vec![xai, ollama]));
    let facade = build_test_facade(AppDependencies::for_tests(
        bundle.memory_repo,
        bundle.vector_store,
        bundle.embedding,
        llm,
        bundle.config,
        Arc::new(orchestrator::NoopEventPublisher),
    ));

    let (memory, _) = facade
        .assimilate("résilience prolongée", None)
        .await
        .expect("Ollama doit absorber la panne xAI");
    assert_eq!(memory.title, "Ollama fallback");
}
