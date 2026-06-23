//! Tests Phase 7 — boucle agent multi-tours, outils Cortex, graphe de connaissances.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use cortex::{Backlink, BacklinkKind, Memory, SearchFilter, SessionKey};
use infrastructure::{FileMemoryRepository, SqliteSessionStore};
use orchestrator::agent::{AgentConfig, AgentLoop, AgentTurnRequest};
use orchestrator::testing::{
    assert_workspace_consistent, build_test_facade, InMemoryDraftRepository,
    InMemoryEmbeddingProvider, InMemoryVectorStore, ToolScriptLlmProvider,
};
use orchestrator::{
    execute_command, AppDependencies, Command, NoopEventPublisher, OrchestratorConfig,
};
use tempfile::tempdir;

fn phase7_config(workspace: &std::path::Path) -> OrchestratorConfig {
    let mut cfg = OrchestratorConfig::default();
    cfg.workspace_root = workspace.to_path_buf();
    cfg.embedding_dim = 8;
    cfg.vector_store.embedding_dimension = 8;
    cfg
}

async fn build_phase7_deps(
    workspace: &std::path::Path,
    llm: Arc<dyn orchestrator::LlmProvider>,
) -> AppDependencies {
    let cfg = phase7_config(workspace);
    std::fs::create_dir_all(cfg.memories_dir()).unwrap();
    std::fs::create_dir_all(workspace.join(".orchestrateur")).unwrap();

    let memory_repo: Arc<dyn cortex::MemoryRepository> =
        Arc::new(FileMemoryRepository::new(cfg.memories_dir()));
    let vector_store: Arc<dyn cortex::VectorStore> = Arc::new(InMemoryVectorStore::new());
    let embedding: Arc<dyn cortex::EmbeddingProvider> =
        Arc::new(InMemoryEmbeddingProvider::new(cfg.embedding_dim));
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
async fn phase7_multi_turn_persists_sessions_and_graph() {
    let dir = tempdir().unwrap();
    let llm = ToolScriptLlmProvider::new(vec![
        "Tour un confirmé.".into(),
        "Tour deux confirmé.".into(),
        "Tour trois confirmé.".into(),
    ]);
    let deps = build_phase7_deps(dir.path(), llm).await;
    let session_key = SessionKey::new("phase7-multi").unwrap();
    let agent = AgentLoop::new(
        deps.clone(),
        AgentConfig {
            auto_assimilate_turn: false,
            message_preprocess: false,
            ..AgentConfig::default()
        },
        None,
    );

    for expected in ["Tour un confirmé.", "Tour deux confirmé.", "Tour trois confirmé."] {
        let result = agent
            .run_turn(AgentTurnRequest {
                session_key: session_key.clone(),
                message: "question".into(),
                personality_prefix: None,
            })
            .await
            .expect("tour agent");
        assert_eq!(result.reply, expected);
    }

    let turns = deps.session_repo.list_turns(&session_key).await.unwrap();
    assert_eq!(turns.len(), 6);
    assert_workspace_consistent(&deps).await.expect("cohérence workspace");
}

#[tokio::test]
async fn phase7_tool_loop_invokes_memory_search() {
    let dir = tempdir().unwrap();
    let tool_response = r#"Je consulte le Cortex.
```tool
{"name":"memory_search","arguments":{"query":"hexagonale","limit":3}}
```
"#;
    let llm = ToolScriptLlmProvider::new(vec![
        tool_response.into(),
        "J'ai trouvé une mémoire sur l'architecture hexagonale.".into(),
    ]);
    let deps = build_phase7_deps(dir.path(), llm.clone()).await;
    let facade = build_test_facade(deps.clone());
    let mem = Memory::new(
        "Architecture hexagonale",
        "Le Cortex isole le domaine des adapters infrastructure.",
    )
    .unwrap();
    facade.save_memory(&mem).await.expect("save memory");

    let agent = AgentLoop::new(
        deps.clone(),
        AgentConfig {
            auto_assimilate_turn: false,
            proactive_memory_search: false,
            message_preprocess: false,
            ..AgentConfig::default()
        },
        None,
    );

    let result = agent
        .run_turn(AgentTurnRequest {
            session_key: SessionKey::new("phase7-tools").unwrap(),
            message: "Que sais-tu sur l'architecture ?".into(),
            personality_prefix: None,
        })
        .await
        .expect("tour avec outil");

    assert_eq!(result.tools_invoked, vec!["memory_search"]);
    assert!(result.reply.contains("architecture hexagonale"));
    assert_eq!(llm.call_count(), 2);
}

#[tokio::test]
async fn phase7_graph_hubs_visible_after_assimilation() {
    let dir = tempdir().unwrap();
    let deps = build_phase7_deps(
        dir.path(),
        Arc::new(orchestrator::testing::InMemoryLlmProvider),
    )
    .await;
    let facade = build_test_facade(deps.clone());

    let hub = Memory::new("Hub central", "Nœud pivot du graphe Phase 7.").unwrap();
    facade.save_memory(&hub).await.expect("save hub");

    let mut spoke = Memory::new("Spoke A", "Mémoire reliée au hub.").unwrap();
    spoke.set_backlinks(vec![Backlink::new(hub.id, 0.85, BacklinkKind::Semantic).unwrap()]);
    facade.save_memory(&spoke).await.expect("save spoke");

    let resp = execute_command(&facade, Command::Graph).await;
    match resp {
        orchestrator::Response::GraphSummary {
            node_count,
            edge_count,
            hubs,
        } => {
            assert_eq!(node_count, 2);
            assert_eq!(edge_count, 1);
            assert!(!hubs.is_empty());
            assert_eq!(hubs[0].memory_id, hub.id);
        }
        other => panic!("réponse graphe inattendue: {other:?}"),
    }

    let hits = facade
        .search_memories("pivot graphe", &SearchFilter::default())
        .await
        .expect("recherche");
    assert!(!hits.is_empty());
}