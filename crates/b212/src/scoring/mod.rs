//! Scoring B212 — Trade Location, Quick Check, Alignment.

mod alignment_score;
mod context;
mod helpers;
mod quick_check;
mod trade_location_score;

pub use alignment_score::compute as compute_alignment_score;
pub use context::ScoringContext;
pub use quick_check::compute as compute_quick_check;
pub use trade_location_score::compute as compute_trade_location_score;

use crate::types::ScoreBundle;

/// Combine les trois scores en bundle final.
#[must_use]
pub fn build_score_bundle(ctx: &ScoringContext<'_>) -> ScoreBundle {
    let trade_location = compute_trade_location_score(ctx);
    let quick_check = compute_quick_check(ctx);
    let alignment = compute_alignment_score(ctx);
    let recommended_sizing = recommend_sizing(&trade_location, &quick_check, &alignment);
    ScoreBundle {
        trade_location,
        quick_check,
        alignment,
        recommended_sizing,
    }
}

fn recommend_sizing(
    tls: &crate::types::TradeLocationScore,
    qc: &crate::types::QuickCheckResult,
    alignment: &crate::types::AlignmentScore,
) -> String {
    if !qc.passed || alignment.total < 5 || tls.total < 6 {
        return "none".into();
    }
    if tls.total >= 8 && alignment.total >= 7 && qc.passed {
        return "normal".into();
    }
    if tls.total >= 6 {
        return "reduced".into();
    }
    "none".into()
}