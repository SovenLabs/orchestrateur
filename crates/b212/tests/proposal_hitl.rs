use std::path::PathBuf;

use b212::{
    approve_proposal, build_narrative, build_setup_analysis, build_trade_proposal,
    mark_sim_executed, reject_proposal, ProposalStatus,
};

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
async fn narrative_has_six_parts() {
    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = b212::ModuleContext::new("BTCUSDT", vec![series]);
    let analysis = build_setup_analysis(&ctx, "london");
    let narrative = build_narrative(&analysis);
    assert!(narrative.contains("1. Macro"));
    assert!(narrative.contains("6. Plan"));
}

#[tokio::test]
async fn hitl_transitions_are_strict() {
    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = b212::ModuleContext::new("BTCUSDT", vec![series]);
    let analysis = build_setup_analysis(&ctx, "london");
    let proposal = match build_trade_proposal(&analysis) {
        Ok(p) => p,
        Err(_) => return,
    };
    let approved = approve_proposal(proposal).unwrap();
    assert_eq!(approved.status, ProposalStatus::HumanApproved);
    let executed = mark_sim_executed(approved).unwrap();
    assert_eq!(executed.status, ProposalStatus::SimExecuted);
}

#[tokio::test]
async fn reject_requires_pending_status() {
    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = b212::ModuleContext::new("BTCUSDT", vec![series]);
    let analysis = build_setup_analysis(&ctx, "london");
    let proposal = match build_trade_proposal(&analysis) {
        Ok(p) => p,
        Err(_) => return,
    };
    let rejected = reject_proposal(proposal, "test reject").unwrap();
    assert_eq!(rejected.status, ProposalStatus::HumanRejected);
    assert_eq!(rejected.reject_reason.as_deref(), Some("test reject"));
    assert!(approve_proposal(rejected).is_err());
}

#[test]
fn trade_proposal_roundtrip_json() {
    let proposal = b212::TradeProposal {
        id: "b212-test".into(),
        symbol: "BTCUSDT".into(),
        session: "london".into(),
        side: "long".into(),
        status: ProposalStatus::PendingHuman,
        trade_location_score: 7,
        quick_check_passed: true,
        alignment_score: 8,
        sizing: "reduced".into(),
        narrative: "test".into(),
        lineage: b212::B212Lineage::fixture("test"),
        created_at: "2026-06-23T00:00:00Z".into(),
        reject_reason: None,
    };
    let json = serde_json::to_string(&proposal).unwrap();
    let back: b212::TradeProposal = serde_json::from_str(&json).unwrap();
    assert_eq!(back, proposal);
}