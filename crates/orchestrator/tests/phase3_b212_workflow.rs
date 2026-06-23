//! Tests E2E Phase 3 — workflow B212, agents domaine, bridge.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::sync::Arc;

use b212::{MarketDataProvider, Timeframe};
use cortex::{EmbeddingProvider, MemoryRepository, SessionRepository, VectorStore};
use infrastructure::{
    FileB212Journal, FileProposalRepository, FileSimTradeRepository, FixtureMarketDataProvider,
};
use orchestrator::testing::{
    InMemoryDraftRepository, InMemoryEmbeddingProvider, InMemoryLlmProvider, InMemoryVectorStore,
};
use orchestrator::{
    ensure_b212_agents, execute_command, B212AnalyzeRequest, B212_AGENTS, Command,
    OrchestratorConfig, OrchestratorFacade,
};
use tempfile::tempdir;

async fn build_phase3_deps(workspace: &std::path::Path) -> orchestrator::AppDependencies {
    let mut cfg = OrchestratorConfig::default();
    cfg.workspace_root = workspace.to_path_buf();
    cfg.embedding_dim = 8;
    cfg.vector_store.embedding_dimension = 8;
    cfg.b212.enabled = true;

    let fixtures_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("workspace")
        .join("b212")
        .join("fixtures");
    let fixtures_dir = workspace.join("b212").join("fixtures");
    std::fs::create_dir_all(&fixtures_dir).unwrap();
    for name in [
        "btc_trend_1h.json",
        "btc_range_4h.json",
        "eth_compression_15m.json",
    ] {
        let src = fixtures_src.join(name);
        assert!(
            src.exists(),
            "fixture source missing: {}",
            src.display()
        );
        std::fs::copy(&src, fixtures_dir.join(name)).unwrap();
    }

    let memory_repo: Arc<dyn MemoryRepository> =
        Arc::new(orchestrator::testing::InMemoryMemoryRepository::new());
    let vector_store: Arc<dyn VectorStore> = Arc::new(InMemoryVectorStore::new());
    let embedding: Arc<dyn EmbeddingProvider> =
        Arc::new(InMemoryEmbeddingProvider::new(cfg.embedding_dim));
    let llm: Arc<dyn orchestrator::LlmProvider> = Arc::new(InMemoryLlmProvider);
    let session_repo: Arc<dyn SessionRepository> = Arc::new(
        infrastructure::SqliteSessionStore::open(&cfg.sessions_db_path()).expect("sqlite"),
    );
    let draft_repo: Arc<dyn orchestrator::DraftRepository> =
        Arc::new(InMemoryDraftRepository::new());

    let market_data: Arc<dyn MarketDataProvider> =
        Arc::new(FixtureMarketDataProvider::new(fixtures_dir));
    let journal: Arc<dyn b212::B212Journal> = Arc::new(FileB212Journal::new(cfg.b212_journal_dir()));
    let proposals: Arc<dyn b212::ProposalRepository> =
        Arc::new(FileProposalRepository::new(cfg.b212_proposals_dir()));
    let sim_trades: Arc<dyn b212::SimTradeRepository> =
        Arc::new(FileSimTradeRepository::new(cfg.b212_sim_dir()));

    std::fs::create_dir_all(cfg.agents_dir()).unwrap();
    std::fs::create_dir_all(workspace.join(".orchestrateur")).unwrap();

    let security = orchestrator::build_test_security_context(&cfg);
    orchestrator::AppDependencies::new(
        memory_repo,
        vector_store,
        embedding,
        llm,
        session_repo,
        draft_repo,
        cfg,
        security,
        None,
        Some(market_data),
        Some(journal),
        Some(proposals),
        Some(sim_trades),
    )
}

#[tokio::test]
async fn phase3_b212_init_agents_creates_six_domain_agents() {
    let dir = tempdir().unwrap();
    let deps = build_phase3_deps(dir.path()).await;
    let facade = OrchestratorFacade::new(deps);
    let manager = facade.agent_manager().await.unwrap();
    let agents = ensure_b212_agents(&manager).await.unwrap();
    assert_eq!(agents.len(), B212_AGENTS.len());
    for def in B212_AGENTS {
        assert!(manager.get(def.id).await.is_ok());
    }
}

#[tokio::test]
async fn phase3_b212_analyze_workflow_returns_six_steps() {
    let dir = tempdir().unwrap();
    let deps = build_phase3_deps(dir.path()).await;
    let facade = OrchestratorFacade::new(deps);
    let result = facade
        .b212_analyze(B212AnalyzeRequest {
            symbol: "BTCUSDT".into(),
            session: "london".into(),
            lookback: 24,
        })
        .await
        .unwrap();
    assert_eq!(result.steps.len(), 6);
    assert_eq!(result.analysis.modules.len(), 5);
    assert!(result.analysis.cardinal.is_some());
    assert!(result.analysis.scores.is_some());
}

#[tokio::test]
async fn phase3_b212_bridge_analyze_command() {
    let dir = tempdir().unwrap();
    let deps = build_phase3_deps(dir.path()).await;
    let facade = OrchestratorFacade::new(deps);
    let resp = execute_command(
        &facade,
        Command::B212Analyze {
            symbol: "BTCUSDT".into(),
            session: "london".into(),
            lookback: 24,
        },
    )
    .await;
    match resp {
        orchestrator::Response::B212Workflow { result } => {
            assert_eq!(result.symbol, "BTCUSDT");
            assert_eq!(result.step_count, 6);
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[tokio::test]
async fn phase3_b212_bridge_init_agents_command() {
    let dir = tempdir().unwrap();
    let deps = build_phase3_deps(dir.path()).await;
    let facade = OrchestratorFacade::new(deps);
    let resp = execute_command(&facade, Command::B212InitAgents).await;
    match resp {
        orchestrator::Response::B212AgentsReady { agent_ids } => {
            assert_eq!(agent_ids.len(), B212_AGENTS.len());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[tokio::test]
async fn phase3_b212_workflow_loads_multi_timeframe_for_btc() {
    let dir = tempdir().unwrap();
    let deps = build_phase3_deps(dir.path()).await;
    let service = orchestrator::B212WorkflowService::new(deps).unwrap();
    let result = service
        .run(B212AnalyzeRequest {
            symbol: "BTCUSDT".into(),
            session: "ny".into(),
            lookback: 32,
        })
        .await
        .unwrap();
    let htf = result
        .analysis
        .modules
        .iter()
        .find(|m| m.module == b212::ModuleId::B2_5);
    assert!(htf.is_some());
    let _ = Timeframe::H4;
}