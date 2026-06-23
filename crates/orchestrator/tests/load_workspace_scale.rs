//! Tests de charge Phase 7 — mémoires et agents à grande échelle.
//!
//! Exécution : `cargo test -p orchestrator --test load_workspace_scale -- --ignored`
//! Voir `tests/load/README.md`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use cortex::KnowledgeGraph;
use orchestrator::testing::{build_test_facade, test_memory, MockBundle};
use orchestrator::{AgentManager, OrchestratorConfig};

#[tokio::test]
#[ignore = "charge: 10 000 mémoires indexées + graphe cohérent"]
async fn load_10k_memories_workspace_consistent() {
    let mut bundle = MockBundle::new();
    bundle.config.embedding_dim = 8;
    bundle.config.vector_store.embedding_dimension = 8;
    let deps = bundle.into_deps();
    let facade = build_test_facade(deps.clone());

    for i in 0..10_000 {
        let mem = test_memory(i).expect("mémoire");
        facade.save_memory(&mem).await.expect("save");
    }

    let memories = deps.memory_repo.list().await.unwrap();
    assert_eq!(memories.len(), 10_000);
    let graph = KnowledgeGraph::from_memories(&memories);
    graph.validate().expect("graphe valide");
}

#[tokio::test]
#[ignore = "charge: 100 agents persistants"]
async fn load_100_persistent_agents() {
    let dir = tempfile::tempdir().unwrap();
    let mut cfg = OrchestratorConfig::default();
    cfg.workspace_root = dir.path().to_path_buf();
    std::fs::create_dir_all(cfg.agents_dir()).unwrap();

    let bundle = MockBundle::new();
    let mut deps = bundle.into_deps();
    deps.config = cfg;

    let manager = AgentManager::new(deps).await.unwrap();
    for i in 0..100 {
        manager
            .create_agent(
                &format!("agent-{i:03}"),
                &format!("Agent {i}"),
                "worker",
                None,
            )
            .await
            .unwrap();
    }

    let listed = manager.list().await.unwrap();
    assert_eq!(listed.len(), 100);
}