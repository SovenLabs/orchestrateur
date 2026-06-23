//! E2E Phase 3 — analyse → approve → sim_execute (journal + fills).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod phase3_b212_helpers;
use phase3_b212_helpers::synthetic_eligible_proposal;

use std::path::PathBuf;
use std::sync::Arc;

use b212::{MarketDataProvider, ProposalStatus};
use cortex::{EmbeddingProvider, MemoryRepository, SessionRepository, VectorStore};
use infrastructure::{
    FileB212Journal, FileProposalRepository, FileSimTradeRepository, FixtureMarketDataProvider,
};
use orchestrator::testing::{
    InMemoryDraftRepository, InMemoryEmbeddingProvider, InMemoryLlmProvider, InMemoryVectorStore,
};
use orchestrator::{
    execute_command, B212WorkflowService, Command, OrchestratorConfig, OrchestratorFacade,
};
use tempfile::tempdir;

async fn build_deps(workspace: &std::path::Path) -> orchestrator::AppDependencies {
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
    for name in ["btc_trend_1h.json", "btc_range_4h.json"] {
        std::fs::copy(fixtures_src.join(name), fixtures_dir.join(name)).unwrap();
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
async fn phase3_b212_hitl_e2e_analyze_approve_sim_execute() {
    let dir = tempdir().unwrap();
    let deps = build_deps(dir.path()).await;
    let facade = OrchestratorFacade::new(deps.clone());
    let gov = B212WorkflowService::new(deps)
        .unwrap()
        .governance()
        .expect("governance");

    let proposal = synthetic_eligible_proposal();
    gov.register_proposal(&proposal).await.unwrap();
    assert_eq!(proposal.status, ProposalStatus::PendingHuman);

    let approved = facade.b212_approve_proposal(&proposal.id).await.unwrap();
    assert_eq!(approved.status, ProposalStatus::HumanApproved);

    let (executed, fill) = facade.b212_sim_execute(&proposal.id).await.unwrap();
    assert_eq!(executed.status, ProposalStatus::SimExecuted);
    assert!(fill.entry_price > 0.0);
    assert_eq!(fill.proposal_id, proposal.id);

    let journal_raw = tokio::fs::read_to_string(dir.path().join("b212/journal/audit.jsonl"))
        .await
        .unwrap();
    assert!(journal_raw.contains("proposal_created"));
    assert!(journal_raw.contains("proposal_approved"));
    assert!(journal_raw.contains("proposal_sim_executed"));
    assert!(journal_raw.contains(&fill.id));

    let fill_path = dir.path().join("b212/sim").join(format!("{}.json", fill.id));
    assert!(fill_path.exists());
}

#[tokio::test]
async fn phase3_b212_bridge_sim_execute_command() {
    let dir = tempdir().unwrap();
    let deps = build_deps(dir.path()).await;
    let facade = OrchestratorFacade::new(deps.clone());
    let gov = B212WorkflowService::new(deps)
        .unwrap()
        .governance()
        .expect("governance");

    let proposal = synthetic_eligible_proposal();
    gov.register_proposal(&proposal).await.unwrap();
    facade.b212_approve_proposal(&proposal.id).await.unwrap();
    let proposal_id = proposal.id;

    let resp = execute_command(&facade, Command::B212SimExecute { id: proposal_id }).await;
    match resp {
        orchestrator::Response::B212SimExecuted { proposal, fill } => {
            assert_eq!(proposal.status, "sim_executed");
            assert!(fill.entry_price > 0.0);
            assert_eq!(fill.realized_pnl_usd, 0.0);
        }
        other => panic!("unexpected response: {other:?}"),
    }
}