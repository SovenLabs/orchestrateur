//! Tests intégration Phase 2b — bridge agents + worker.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use orchestrator::testing::{
    InMemoryDraftRepository, InMemoryEmbeddingProvider, InMemoryLlmProvider,
    InMemoryVectorStore,
};
use orchestrator::{
    execute_command, AppDependencies, Command, NoopEventPublisher, OrchestratorConfig,
    OrchestratorFacade, Response,
};
use tempfile::tempdir;

async fn test_facade(workspace: &std::path::Path) -> OrchestratorFacade {
    let mut cfg = OrchestratorConfig::default();
    cfg.workspace_root = workspace.to_path_buf();
    cfg.embedding_dim = 8;
    cfg.agents.enabled = true;
    std::fs::create_dir_all(cfg.agents_dir()).unwrap();

    let deps = AppDependencies::for_tests(
        Arc::new(orchestrator::testing::InMemoryMemoryRepository::new()),
        Arc::new(InMemoryVectorStore::new()),
        Arc::new(InMemoryEmbeddingProvider::new(8)),
        Arc::new(InMemoryLlmProvider),
        Arc::new(orchestrator::testing::InMemorySessionRepository::new()),
        Arc::new(InMemoryDraftRepository::new()),
        cfg,
        Arc::new(NoopEventPublisher),
    );
    OrchestratorFacade::new(deps)
}

#[tokio::test]
async fn phase2b_bridge_create_and_turn_agent() {
    let dir = tempdir().unwrap();
    let facade = test_facade(dir.path()).await;

    let created = execute_command(
        &facade,
        Command::CreateAgent {
            id: "bridge-bot".into(),
            name: "Bridge Bot".into(),
            role: "test".into(),
            model: None,
        },
    )
    .await;
    assert!(matches!(created, Response::AgentDetail { .. }));

    let turn = execute_command(
        &facade,
        Command::AgentTurn {
            id: "bridge-bot".into(),
            message: "Hello bridge".into(),
        },
    )
    .await;
    match turn {
        Response::AgentTurnReply { reply, .. } => assert_eq!(reply, "Hello bridge"),
        other => panic!("réponse inattendue: {other:?}"),
    }

    let list = execute_command(&facade, Command::ListAgents).await;
    match list {
        Response::AgentList { items } => assert_eq!(items.len(), 1),
        other => panic!("liste inattendue: {other:?}"),
    }
}

#[tokio::test]
async fn phase2b_bridge_inter_agent_message() {
    let dir = tempdir().unwrap();
    let facade = test_facade(dir.path()).await;

    for id in ["alpha", "beta"] {
        execute_command(
            &facade,
            Command::CreateAgent {
                id: id.into(),
                name: id.into(),
                role: "test".into(),
                model: None,
            },
        )
        .await;
    }

    let sent = execute_command(
        &facade,
        Command::AgentSendMessage {
            from: "alpha".into(),
            to: "beta".into(),
            body: "ping beta".into(),
        },
    )
    .await;
    assert!(matches!(sent, Response::AgentMessageSent { .. }));

    let inbox = execute_command(
        &facade,
        Command::AgentMessages {
            id: "beta".into(),
            mark_read: false,
        },
    )
    .await;
    match inbox {
        Response::AgentMessages { items } => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].body, "ping beta");
        }
        other => panic!("inbox inattendue: {other:?}"),
    }
}