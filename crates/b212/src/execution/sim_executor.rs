//! SimExecutor paper — fill au prix fixture, persistance via port.

use chrono::Utc;
use uuid::Uuid;

use crate::error::B212Error;
use crate::types::{SimFill, TradeProposal};

/// Notionnel USD selon le sizing Bible.
#[must_use]
pub fn notional_for_sizing(sizing: &str, base_notional_usd: f64) -> f64 {
    match sizing {
        "reduced" => base_notional_usd * 0.5,
        "normal" => base_notional_usd,
        _ => base_notional_usd * 0.25,
    }
}

/// Simule un fill paper à `entry_price` pour une proposition approuvée.
///
/// # Errors
///
/// Retourne [`B212Error::SimExecution`] si le prix ou le sizing est invalide.
pub fn execute_paper_fill(
    proposal: &TradeProposal,
    entry_price: f64,
    base_notional_usd: f64,
) -> Result<SimFill, B212Error> {
    if entry_price <= 0.0 {
        return Err(B212Error::SimExecution(format!(
            "prix d'entrée invalide: {entry_price}"
        )));
    }
    if proposal.side == "observe" {
        return Err(B212Error::SimExecution(
            "impossible d'exécuter une proposition observe".into(),
        ));
    }

    let notional_usd = notional_for_sizing(&proposal.sizing, base_notional_usd);
    let quantity = notional_usd / entry_price;

    Ok(SimFill {
        id: format!("sim-{}", Uuid::now_v7()),
        proposal_id: proposal.id.clone(),
        symbol: proposal.symbol.clone(),
        side: proposal.side.clone(),
        entry_price,
        quantity,
        notional_usd,
        realized_pnl_usd: 0.0,
        fill_ts: Utc::now().to_rfc3339(),
        lineage: proposal.lineage.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{B212Lineage, ProposalStatus};

    fn sample_proposal() -> TradeProposal {
        TradeProposal {
            id: "b212-test".into(),
            symbol: "BTCUSDT".into(),
            session: "london".into(),
            side: "long".into(),
            status: ProposalStatus::HumanApproved,
            trade_location_score: 7,
            quick_check_passed: true,
            alignment_score: 6,
            sizing: "normal".into(),
            narrative: "test".into(),
            lineage: B212Lineage::fixture("test"),
            created_at: Utc::now().to_rfc3339(),
            reject_reason: None,
        }
    }

    #[test]
    fn paper_fill_computes_quantity_from_notional() {
        let fill = execute_paper_fill(&sample_proposal(), 50_000.0, 10_000.0).unwrap();
        assert_eq!(fill.notional_usd, 10_000.0);
        assert!((fill.quantity - 0.2).abs() < f64::EPSILON);
        assert_eq!(fill.realized_pnl_usd, 0.0);
    }

    #[test]
    fn reduced_sizing_halves_notional() {
        let mut p = sample_proposal();
        p.sizing = "reduced".into();
        let fill = execute_paper_fill(&p, 40_000.0, 10_000.0).unwrap();
        assert_eq!(fill.notional_usd, 5_000.0);
    }

    #[test]
    fn rejects_invalid_price() {
        let err = execute_paper_fill(&sample_proposal(), 0.0, 10_000.0).unwrap_err();
        assert!(matches!(err, B212Error::SimExecution(_)));
    }
}