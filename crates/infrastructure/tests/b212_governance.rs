use std::path::PathBuf;
use std::sync::Arc;

use b212::{build_setup_analysis, ModuleContext, ProposalStatus};
use infrastructure::{FileB212Journal, FileProposalRepository};
use orchestrator::b212::B212GovernanceService;

async fn load_fixture(name: &str) -> b212::OhlcvSeries {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("workspace")
        .join("b212")
        .join("fixtures")
        .join(name);
    let raw = tokio::fs::read_to_string(&path).await.unwrap();
    serde_json::from_str(&raw).unwrap()
}

#[tokio::test]
async fn governance_journals_analysis_and_persists_proposal() {
    let temp = tempfile::tempdir().unwrap();
    let journal_dir = temp.path().join("journal");
    let proposals_dir = temp.path().join("proposals");

    let journal = Arc::new(FileB212Journal::new(&journal_dir));
    let proposals = Arc::new(FileProposalRepository::new(&proposals_dir));
    let service = B212GovernanceService::new(journal, proposals);

    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let analysis = build_setup_analysis(&ctx, "london");

    let created = service.process_analysis(&analysis).await.unwrap();
    let journal_path = journal_dir.join("audit.jsonl");
    assert!(journal_path.exists());
    let journal_raw = tokio::fs::read_to_string(&journal_path).await.unwrap();
    assert!(journal_raw.contains("setup_analyzed"));

    if let Some(proposal) = created {
        assert_eq!(proposal.status, ProposalStatus::PendingHuman);
        assert!(journal_raw.contains("proposal_created"));
        let pending = service.list_pending().await.unwrap();
        assert!(pending.iter().any(|p| p.id == proposal.id));

        let approved = service.approve(&proposal.id).await.unwrap();
        assert_eq!(approved.status, ProposalStatus::HumanApproved);
        let executed = service.sim_execute(&proposal.id).await.unwrap();
        assert_eq!(executed.status, ProposalStatus::SimExecuted);
    }
}

#[tokio::test]
async fn governance_reject_writes_journal() {
    let temp = tempfile::tempdir().unwrap();
    let journal_dir = temp.path().join("journal");
    let proposals_dir = temp.path().join("proposals");

    let journal = Arc::new(FileB212Journal::new(&journal_dir));
    let proposals = Arc::new(FileProposalRepository::new(&proposals_dir));
    let service = B212GovernanceService::new(journal, proposals);

    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let analysis = build_setup_analysis(&ctx, "london");

    if let Some(proposal) = service.process_analysis(&analysis).await.unwrap() {
        let rejected = service.reject(&proposal.id, "no edge").await.unwrap();
        assert_eq!(rejected.status, ProposalStatus::HumanRejected);
        let journal_raw = tokio::fs::read_to_string(journal_dir.join("audit.jsonl"))
            .await
            .unwrap();
        assert!(journal_raw.contains("proposal_rejected"));
    }
}