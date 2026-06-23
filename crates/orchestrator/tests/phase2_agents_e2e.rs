//! Tests E2E Phase 2 — agents persistants, registre, heartbeat, messagerie.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use cortex::{ConversationTurn, TurnRole};
use infrastructure::SqliteSessionStore;
use orchestrator::{AgentIdentity, AgentStatus};
use orchestrator::testing::{
    InMemoryDraftRepository, InMemoryEmbeddingProvider, InMemoryLlmProvider,
    InMemoryVectorStore,
};
use orchestrator::{
    AgentManager, AppDependencies, NoopEventPublisher, OrchestratorConfig,
};
use tempfile::tempdir;

async fn build_phase2_deps(workspace: &std::path::Path) -> AppDependencies {
    let mut cfg = OrchestratorConfig::default();
    cfg.workspace_root = workspace.to_path_buf();
    cfg.embedding_dim = 8;
    cfg.vector_store.embedding_dimension = 8;

    std::fs::create_dir_all(cfg.agents_dir()).unwrap();
    std::fs::create_dir_all(workspace.join(".orchestrateur")).unwrap();

    let memory_repo: Arc<dyn cortex::MemoryRepository> =
        Arc::new(orchestrator::testing::InMemoryMemoryRepository::new());
    let vector_store: Arc<dyn cortex::VectorStore> = Arc::new(InMemoryVectorStore::new());
    let embedding: Arc<dyn cortex::EmbeddingProvider> =
        Arc::new(InMemoryEmbeddingProvider::new(cfg.embedding_dim));
    let llm: Arc<dyn orchestrator::LlmProvider> = Arc::new(InMemoryLlmProvider);
    let session_repo: Arc<dyn cortex::SessionRepository> = Arc::new(
        SqliteSessionStore::open(&cfg.sessions_db_path()).expect("sqlite sessions"),
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
async fn phase2_create_agent_scaffolds_folder_and_registry() {
    let dir = tempdir().unwrap();
    let deps = build_phase2_deps(dir.path()).await;
    let manager = AgentManager::new(deps).await.unwrap();

    let agent = manager
        .create_agent("researcher", "Chercheur", "analyste", Some("grok-4.3"))
        .await
        .unwrap();

    assert!(agent.root.join("personality.md").exists());
    assert!(agent.root.join("heartbeat.md").exists());
    assert!(agent.root.join("config.toml").exists());
    assert!(agent.root.join("tasks").is_dir());
    assert!(agent.root.join("memories").is_dir());
    assert!(agent.root.join("messages/inbox").is_dir());
    assert!(agent.root.join("messages/outbox").is_dir());
    assert_eq!(agent.status(), AgentStatus::Sleeping);

    let registry_path = dir.path().join("registry/AGENTS_REGISTRY.md");
    assert!(registry_path.exists());
    let registry_md = std::fs::read_to_string(registry_path).unwrap();
    assert!(registry_md.contains("researcher"));
    assert!(registry_md.contains("Chercheur"));

    let listed = manager.list().await.unwrap();
    assert_eq!(listed.len(), 1);
}

#[tokio::test]
async fn phase2_wake_sleep_lifecycle_persists_status() {
    let dir = tempdir().unwrap();
    let deps = build_phase2_deps(dir.path()).await;
    let manager = AgentManager::new(deps.clone()).await.unwrap();

    manager
        .create_agent("worker", "Worker", "exécutant", None)
        .await
        .unwrap();

    let awake = manager.wake("worker").await.unwrap();
    assert_eq!(awake.status(), AgentStatus::Awake);

    let key = awake.session_key().unwrap();
    deps.session_repo
        .append_turn(
            &key,
            ConversationTurn::new(TurnRole::User, "ping"),
        )
        .await
        .unwrap();
    let turns = deps.session_repo.list_turns(&key).await.unwrap();
    assert_eq!(turns.len(), 1);

    let sleeping = manager.sleep("worker").await.unwrap();
    assert_eq!(sleeping.status(), AgentStatus::Sleeping);

    let reloaded = AgentManager::new(deps).await.unwrap();
    let agent = reloaded.get("worker").await.unwrap();
    assert_eq!(agent.status(), AgentStatus::Sleeping);
}

#[tokio::test]
async fn phase2_inter_agent_messaging_inbox_outbox() {
    let dir = tempdir().unwrap();
    let deps = build_phase2_deps(dir.path()).await;
    let manager = AgentManager::new(deps).await.unwrap();

    manager
        .create_agent("alpha", "Alpha", "coordinateur", None)
        .await
        .unwrap();
    manager
        .create_agent("beta", "Beta", "exécutant", None)
        .await
        .unwrap();

    let msg = manager
        .send_message("alpha", "beta", "Bonjour Beta, mission Phase 2.")
        .await
        .unwrap();
    assert_eq!(msg.from, "alpha");
    assert_eq!(msg.to, "beta");

    let inbox = manager.receive_messages("beta", false).await.unwrap();
    assert_eq!(inbox.len(), 1);
    assert_eq!(inbox[0].body, "Bonjour Beta, mission Phase 2.");

    let report = manager.background("beta").await.unwrap();
    assert_eq!(report.inbox_count, 1);
    assert!(report.executed.iter().any(|e| e.starts_with("check_inbox:")));
}

#[tokio::test]
async fn phase2_agent_memory_assimilation_to_scoped_folder() {
    let dir = tempdir().unwrap();
    let deps = build_phase2_deps(dir.path()).await;
    let manager = AgentManager::new(deps).await.unwrap();

    let agent = manager
        .create_agent("scribe", "Scribe", "archiviste", None)
        .await
        .unwrap();

    manager
        .assimilate_memory(
            "scribe",
            "Note agent : préférence pour les résumés structurés en Markdown.",
            &["agent-note".into()],
        )
        .await
        .unwrap();

    let md_files: Vec<_> = std::fs::read_dir(agent.root.join("memories"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
        .collect();
    assert_eq!(md_files.len(), 1);
}