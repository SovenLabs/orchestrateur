//! Trade Location Score /10 (Bible B2).

use crate::types::{ModuleId, TradeLocationScore};

use super::context::ScoringContext;
use super::helpers::{b12_validation, liquidity_proximity_ratio, tf_alignment_score};

fn tls_sizing(total: u8) -> &'static str {
    if total >= 8 {
        "normal"
    } else if total >= 6 {
        "reduced"
    } else {
        "none"
    }
}

/// Calcule le Trade Location Score.
#[must_use]
pub fn compute(ctx: &ScoringContext<'_>) -> TradeLocationScore {
    let liquidity_proximity = score_liquidity_proximity(ctx);
    let htf_ltf_confluence = score_htf_ltf_confluence(ctx);
    let extreme_or_retest = score_extreme_or_retest(ctx);
    let b12_validation_pts = score_b12_validation(ctx);
    let total = liquidity_proximity
        .saturating_add(htf_ltf_confluence)
        .saturating_add(extreme_or_retest)
        .saturating_add(b12_validation_pts)
        .min(10);
    let sizing = tls_sizing(total).to_string();
    let rationale = format!(
        "TLS {total}/10 : liquidité {liquidity_proximity}/3, confluence {htf_ltf_confluence}/3, \
         extrémité {extreme_or_retest}/2, B12 {b12_validation_pts}/2 → taille {sizing}."
    );
    TradeLocationScore {
        total,
        liquidity_proximity,
        htf_ltf_confluence,
        extreme_or_retest,
        b12_validation: b12_validation_pts,
        sizing,
        rationale,
    }
}

fn score_liquidity_proximity(ctx: &ScoringContext<'_>) -> u8 {
    let ratio = liquidity_proximity_ratio(ctx);
    if ratio >= 0.9 {
        3
    } else if ratio >= 0.5 {
        2
    } else if ctx.any_signal_triggered() || ratio > 0.0 {
        1
    } else {
        0
    }
}

fn score_htf_ltf_confluence(ctx: &ScoringContext<'_>) -> u8 {
    let score = tf_alignment_score(ctx);
    if score >= 80 {
        3
    } else if score >= 55 {
        2
    } else if score >= 40 {
        1
    } else {
        0
    }
}

fn score_extreme_or_retest(ctx: &ScoringContext<'_>) -> u8 {
    let payload = match ctx.module_payload(ModuleId::B2) {
        Some(p) => p,
        None => return 0,
    };
    let bos = payload.get("bos").and_then(|v| v.as_bool()).unwrap_or(false);
    let trade_exists = payload
        .get("trade_exists")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let fvg_count = payload
        .get("fvg_midpoints")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    if trade_exists && bos {
        2
    } else if bos || fvg_count > 0 {
        1
    } else {
        0
    }
}

fn score_b12_validation(ctx: &ScoringContext<'_>) -> u8 {
    match b12_validation(ctx).as_str() {
        "absorption" | "acceptance_above_value" | "rejection_below_value" => 2,
        "rotation_at_poc" => 1,
        _ => 0,
    }
}