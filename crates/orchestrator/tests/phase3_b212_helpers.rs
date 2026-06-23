//! Helpers partagés tests Phase 3 B212.

#![allow(dead_code)]

use std::path::PathBuf;

use b212::{B212Lineage, ProposalStatus, TradeProposal};
use chrono::Utc;
use uuid::Uuid;

pub async fn load_fixture(name: &str) -> b212::OhlcvSeries {
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

/// Proposition HITL synthétique éligible à l'exécution paper.
pub fn synthetic_eligible_proposal() -> TradeProposal {
    TradeProposal {
        id: format!("b212-{}", Uuid::now_v7()),
        symbol: "BTCUSDT".into(),
        session: "london".into(),
        side: "long".into(),
        status: ProposalStatus::PendingHuman,
        trade_location_score: 7,
        quick_check_passed: true,
        alignment_score: 7,
        sizing: "reduced".into(),
        narrative: "Proposition E2E Phase 3 B212.".into(),
        lineage: B212Lineage::fixture("phase3_e2e"),
        created_at: Utc::now().to_rfc3339(),
        reject_reason: None,
    }
}