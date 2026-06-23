//! Tests Phase 4 — suppression agent (registre + bridge).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use orchestrator::testing::{
    InMemoryDraftRepository, InMemoryEmbeddingProvider, InMemoryLlmProvider,
    InMemoryVectorStore,
};
use orchestrator::{
    execute_command, AgentManager, AppDependencies, Command, NoopEventPublisher,
    OrchestratorConfig, OrchestratorFacade, Response,
};
use tempfile::tempdir;

async fn build_deps(workspace: &std::path::Path) -> AppDependencies {
    let mut cfg = OrchestratorConfig::default();
    cfg.workspace_root = workspace.to_path_buf();
    cfg.embedding_dim = 8;
    cfg.vector_store.embedding_dimension = 8;
    std::fs::create_dir_all(cfg.agents_dir()).unwrap();

    AppDependencies::for_tests(
        Arc::new(orchestrator::testing::InMemoryMemoryRepository::new()),
        Arc::new(InMemoryVectorStore::new()),
        Arc::new(InMemoryEmbeddingProvider::new(8)),
        Arc::new(InMemoryLlmProvider),
        Arc::new(orchestrator::testing::InMemorySessionRepository::new()),
        Arc::new(InMemoryDraftRepository::new()),
        cfg,
        Arc::new(NoopEventPublisher),
    )
}

#[tokio::test]
async fn phase4_delete_agent_removes_folder_and_registry() {
    let dir = tempdir().unwrap();
    let deps = build_deps(dir.path()).await;
    let manager = AgentManager::new(deps.clone()).await.unwrap();

    let agent = manager
        .create_agent("temp-bot", "Temp", "test", None)
        .await
        .unwrap();
    let root = agent.root.clone();
    assert!(root.exists());

    manager.delete_agent("temp-bot").await.unwrap();
    assert!(!root.exists());

    let listed = manager.list().await.unwrap();
    assert!(listed.is_empty());

    let registry_path = deps.config.agents_registry_path();
    let registry_md = std::fs::read_to_string(registry_path).unwrap();
    assert!(!registry_md.contains("temp-bot"));
}

#[tokio::test]
async fn phase4_bridge_agent_delete_command() {
    let dir = tempdir().unwrap();
    let deps = build_deps(dir.path()).await;
    let facade = OrchestratorFacade::new(deps);

    execute_command(
        &facade,
        Command::CreateAgent {
            id: "bridge-temp".into(),
            name: "Bridge Temp".into(),
            role: "test".into(),
            model: None,
        },
    )
    .await;

    let deleted = execute_command(
        &facade,
        Command::AgentDelete {
            id: "bridge-temp".into(),
        },
    )
    .await;
    match deleted {
        Response::AgentDeleted { id } => assert_eq!(id, "bridge-temp"),
        other => panic!("réponse inattendue: {other:?}"),
    }

    let list = execute_command(&facade, Command::ListAgents).await;
    match list {
        Response::AgentList { items } => assert!(items.is_empty()),
        other => panic!("liste inattendue: {other:?}"),
    }
}

#[tokio::test]
async fn phase4_status_snapshots_count_unread_inbox() {
    let dir = tempdir().unwrap();
    let deps = build_deps(dir.path()).await;
    let manager = AgentManager::new(deps).await.unwrap();

    manager
        .create_agent("sender", "Sender", "test", None)
        .await
        .unwrap();
    manager
        .create_agent("receiver", "Receiver", "test", None)
        .await
        .unwrap();
    manager
        .send_message("sender", "receiver", "ping phase 4")
        .await
        .unwrap();

    let snapshots = manager.status_snapshots(Some("receiver")).await.unwrap();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].unread_inbox, 1);
}