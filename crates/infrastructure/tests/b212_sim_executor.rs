use std::path::PathBuf;
use std::sync::Arc;

use b212::{approve_proposal, ProposalRepository, ProposalStatus, SimTradeRepository};
use infrastructure::{
    FileB212Journal, FileProposalRepository, FileSimTradeRepository, FixtureMarketDataProvider,
};
use orchestrator::b212::{B212GovernanceService, B212SimExecutorService};

#[tokio::test]
async fn sim_executor_persists_fill_and_updates_proposal() {
    let temp = tempfile::tempdir().unwrap();
    let journal = Arc::new(FileB212Journal::new(temp.path().join("journal")));
    let proposals = Arc::new(FileProposalRepository::new(temp.path().join("proposals")));
    let sim_trades = Arc::new(FileSimTradeRepository::new(temp.path().join("sim")));
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("workspace")
        .join("b212")
        .join("fixtures");
    let market_data: Arc<dyn b212::MarketDataProvider> =
        Arc::new(FixtureMarketDataProvider::new(fixtures_dir));

    let gov = B212GovernanceService::new(journal.clone(), proposals.clone());
    let service = B212SimExecutorService::new(gov, market_data, sim_trades.clone(), 10_000.0);

    let proposal = b212::TradeProposal {
        id: "b212-sim-test".into(),
        symbol: "BTCUSDT".into(),
        session: "london".into(),
        side: "long".into(),
        status: ProposalStatus::PendingHuman,
        trade_location_score: 7,
        quick_check_passed: true,
        alignment_score: 7,
        sizing: "reduced".into(),
        narrative: "sim executor test".into(),
        lineage: b212::B212Lineage::fixture("sim_test"),
        created_at: "2026-06-23T00:00:00Z".into(),
        reject_reason: None,
    };
    let approved = approve_proposal(proposal).unwrap();
    proposals.save(&approved).await.unwrap();

    let (executed, fill) = service.execute(&approved.id).await.unwrap();
    assert_eq!(executed.status, ProposalStatus::SimExecuted);
    assert_eq!(fill.proposal_id, approved.id);
    assert!(fill.entry_price > 40_000.0);

    let stored = sim_trades.get(&fill.id).await.unwrap();
    assert!((stored.quantity - fill.quantity).abs() < 1e-9);

    let journal_raw = tokio::fs::read_to_string(temp.path().join("journal/audit.jsonl"))
        .await
        .unwrap();
    assert!(journal_raw.contains("proposal_sim_executed"));
    assert!(journal_raw.contains(&fill.id));
}