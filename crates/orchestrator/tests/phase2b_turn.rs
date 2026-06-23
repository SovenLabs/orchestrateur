//! Tests E2E Phase 2b — tour agent persistant.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use cortex::{SessionRepository, TurnRole};
use infrastructure::SqliteSessionStore;
use orchestrator::testing::{
    InMemoryDraftRepository, InMemoryEmbeddingProvider, InMemoryLlmProvider,
    InMemoryVectorStore,
};
use orchestrator::{AgentManager, AppDependencies, NoopEventPublisher, OrchestratorConfig};
use tempfile::tempdir;

async fn build_deps(workspace: &std::path::Path) -> AppDependencies {
    let mut cfg = OrchestratorConfig::default();
    cfg.workspace_root = workspace.to_path_buf();
    cfg.embedding_dim = 8;
    cfg.vector_store.embedding_dimension = 8;

    std::fs::create_dir_all(cfg.agents_dir()).unwrap();

    let memory_repo: Arc<dyn cortex::MemoryRepository> =
        Arc::new(orchestrator::testing::InMemoryMemoryRepository::new());
    let vector_store: Arc<dyn cortex::VectorStore> = Arc::new(InMemoryVectorStore::new());
    let embedding: Arc<dyn cortex::EmbeddingProvider> =
        Arc::new(InMemoryEmbeddingProvider::new(cfg.embedding_dim));
    let llm: Arc<dyn orchestrator::LlmProvider> = Arc::new(InMemoryLlmProvider);
    let session_repo: Arc<dyn cortex::SessionRepository> = Arc::new(
        SqliteSessionStore::open(&cfg.sessions_db_path()).expect("sqlite"),
    );
    let draft_repo: Arc<dyn orchestrator::DraftRepository> =
        Arc::new(InMemoryDraftRepository::new());

    AppDependencies::for_tests(
        memory_repo,
        vector_store,
        embedding,
        llm,
        session_repo,
        draft_repo,
        cfg,
        Arc::new(NoopEventPublisher),
    )
}

#[tokio::test]
async fn phase2b_persistent_agent_turn_uses_dedicated_session() {
    let dir = tempdir().unwrap();
    let deps = build_deps(dir.path()).await;
    let manager = AgentManager::new(deps.clone()).await.unwrap();

    manager
        .create_agent("scribe", "Scribe", "archiviste", None)
        .await
        .unwrap();

    let result = manager
        .run_turn("scribe", "Note persistante Phase 2b.", None)
        .await
        .unwrap();

    assert_eq!(result.reply, "Note persistante Phase 2b.");
    assert_eq!(result.session_key.as_str(), "agent-scribe");

    let turns = deps
        .session_repo
        .list_turns(&result.session_key)
        .await
        .unwrap();
    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0].role, TurnRole::User);
    assert_eq!(turns[1].role, TurnRole::Assistant);
}