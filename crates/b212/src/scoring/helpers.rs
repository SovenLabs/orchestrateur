//! Helpers scoring à partir des payloads modules.

use crate::types::ModuleId;

use super::context::ScoringContext;

/// Climat macro B1 (`favorable`, `neutre`, `hostile`).
#[must_use]
pub fn macro_climate(ctx: &ScoringContext<'_>) -> String {
    ctx.module_payload(ModuleId::B1)
        .and_then(|p| p.get("climate"))
        .and_then(|v| v.as_str())
        .unwrap_or("neutre")
        .to_string()
}

/// Régime B1.5.
#[must_use]
pub fn market_regime(ctx: &ScoringContext<'_>) -> String {
    ctx.module_payload(ModuleId::B1_5)
        .and_then(|p| p.get("regime"))
        .and_then(|v| v.as_str())
        .unwrap_or("range")
        .to_string()
}

/// Biais structurel B2.
#[must_use]
pub fn structure_bias(ctx: &ScoringContext<'_>) -> String {
    ctx.module_payload(ModuleId::B2)
        .and_then(|p| p.get("bias"))
        .and_then(|v| v.as_str())
        .unwrap_or("neutral")
        .to_string()
}

/// Score alignement timeframe B2.5 (0–100).
#[must_use]
pub fn tf_alignment_score(ctx: &ScoringContext<'_>) -> u8 {
    ctx.module_payload(ModuleId::B2_5)
        .and_then(|p| p.get("alignment_score"))
        .and_then(|v| v.as_u64())
        .map(|v| v.min(100) as u8)
        .unwrap_or(0)
}

/// Contre-tendance HTF vs LTF.
#[must_use]
pub fn counter_trend(ctx: &ScoringContext<'_>) -> bool {
    ctx.module_payload(ModuleId::B2_5)
        .and_then(|p| p.get("counter_trend"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Validation B12.
#[must_use]
pub fn b12_validation(ctx: &ScoringContext<'_>) -> String {
    ctx.module_payload(ModuleId::B12)
        .and_then(|p| p.get("validation"))
        .and_then(|v| v.as_str())
        .unwrap_or("inconclusive")
        .to_string()
}

/// Proximité prix / pools de liquidité (ratio 0–1).
#[must_use]
pub fn liquidity_proximity_ratio(ctx: &ScoringContext<'_>) -> f64 {
    let payload = match ctx.module_payload(ModuleId::B2) {
        Some(p) => p,
        None => return 0.0,
    };
    let pools = payload
        .get("liquidity_pools")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_f64())
                .collect::<Vec<f64>>()
        })
        .unwrap_or_default();
    if pools.is_empty() {
        return 0.0;
    }
    let price = ctx
        .primary_series()
        .and_then(|s| s.bars.last())
        .map(|b| b.close)
        .unwrap_or(0.0);
    if price.abs() < f64::EPSILON {
        return 0.0;
    }
    let min_dist = pools
        .iter()
        .map(|lvl| (price - lvl).abs() / price)
        .fold(f64::INFINITY, f64::min);
    if min_dist <= 0.005 {
        1.0
    } else if min_dist <= 0.015 {
        0.66
    } else if min_dist <= 0.03 {
        0.33
    } else {
        0.0
    }
}

/// Session active (proxy exécution).
#[must_use]
pub fn session_active(session: &str) -> bool {
    matches!(
        session.to_ascii_lowercase().as_str(),
        "london" | "ny" | "new_york" | "asia" | "overlap"
    )
}

/// Bloc Quick Check validé si majorité des checks passent.
#[must_use]
pub fn block_passes(checks: &[crate::types::QuickCheckItem]) -> bool {
    if checks.is_empty() {
        return false;
    }
    let passed = checks.iter().filter(|c| c.passed).count();
    passed * 2 >= checks.len()
}