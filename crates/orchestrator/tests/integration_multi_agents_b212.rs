//! Tests d'intégration Phase 7 — scénarios multi-agents + B212.
//!
//! Voir aussi `tests/integration/README.md` à la racine du dépôt.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::sync::Arc;

use cortex::{ConversationTurn, TurnRole};
use infrastructure::{
    FileB212Journal, FileProposalRepository, FileSimTradeRepository, FixtureMarketDataProvider,
    SqliteSessionStore,
};
use orchestrator::testing::{
    InMemoryDraftRepository, InMemoryEmbeddingProvider, InMemoryLlmProvider, InMemoryVectorStore,
};
use orchestrator::{
    ensure_b212_agents, execute_command, AgentManager, AppDependencies, B212AnalyzeRequest,
    B212_AGENTS, Command, OrchestratorConfig, OrchestratorFacade,
};
use tempfile::tempdir;

async fn build_integration_deps(workspace: &std::path::Path) -> AppDependencies {
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
        if src.exists() {
            std::fs::copy(&src, fixtures_dir.join(name)).unwrap();
        }
    }

    std::fs::create_dir_all(cfg.agents_dir()).unwrap();
    std::fs::create_dir_all(workspace.join(".orchestrateur")).unwrap();

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

    let market_data: Arc<dyn b212::MarketDataProvider> =
        Arc::new(FixtureMarketDataProvider::new(fixtures_dir));
    let journal: Arc<dyn b212::B212Journal> = Arc::new(FileB212Journal::new(cfg.b212_journal_dir()));
    let proposals: Arc<dyn b212::ProposalRepository> =
        Arc::new(FileProposalRepository::new(cfg.b212_proposals_dir()));
    let sim_trades: Arc<dyn b212::SimTradeRepository> =
        Arc::new(FileSimTradeRepository::new(cfg.b212_sim_dir()));

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
async fn integration_multi_agents_messaging_and_b212_coexist() {
    let dir = tempdir().unwrap();
    let deps = build_integration_deps(dir.path()).await;
    let facade = OrchestratorFacade::new(deps.clone());
    let manager = facade.agent_manager().await.unwrap();

    manager
        .create_agent("analyst", "Analyste", "recherche", None)
        .await
        .unwrap();
    manager
        .create_agent("trader", "Trader", "exécution", None)
        .await
        .unwrap();

    let b212_agents = ensure_b212_agents(&manager).await.unwrap();
    assert_eq!(b212_agents.len(), B212_AGENTS.len());

    manager
        .send_message("analyst", "trader", "Signal BTC — revoir la proposition B212.")
        .await
        .unwrap();

    let inbox = manager.receive_messages("trader", false).await.unwrap();
    assert_eq!(inbox.len(), 1);
    assert!(inbox[0].body.contains("B212"));

    let workflow = facade
        .b212_analyze(B212AnalyzeRequest {
            symbol: "BTCUSDT".into(),
            session: "london".into(),
            lookback: 24,
        })
        .await
        .unwrap();
    assert_eq!(workflow.steps.len(), 6);

    let resp = execute_command(
        &facade,
        Command::AgentMessages {
            id: "trader".into(),
            mark_read: false,
        },
    )
    .await;
    match resp {
        orchestrator::Response::AgentMessages { items } => assert!(!items.is_empty()),
        other => panic!("inbox inattendue: {other:?}"),
    }
}

#[tokio::test]
async fn integration_persistent_agents_share_session_store() {
    let dir = tempdir().unwrap();
    let deps = build_integration_deps(dir.path()).await;
    let manager = AgentManager::new(deps.clone()).await.unwrap();

    let a = manager
        .create_agent("alpha", "Alpha", "coordination", None)
        .await
        .unwrap();
    let b = manager
        .create_agent("beta", "Beta", "coordination", None)
        .await
        .unwrap();

    let key_a = a.session_key().unwrap();
    let key_b = b.session_key().unwrap();
    assert_ne!(key_a, key_b);

    deps.session_repo
        .append_turn(&key_a, ConversationTurn::new(TurnRole::User, "ping alpha"))
        .await
        .unwrap();
    deps.session_repo
        .append_turn(&key_b, ConversationTurn::new(TurnRole::User, "ping beta"))
        .await
        .unwrap();

    let turns_a = deps.session_repo.list_turns(&key_a).await.unwrap();
    let turns_b = deps.session_repo.list_turns(&key_b).await.unwrap();
    assert_eq!(turns_a.len(), 1);
    assert_eq!(turns_b.len(), 1);
}