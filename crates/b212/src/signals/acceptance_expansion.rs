//! Acceptance Expansion — le marché habite la nouvelle value.

use crate::types::{B212Lineage, ModuleId, SignalKind, SignalOutput};

use super::context::SignalContext;
use super::helpers::{
    avg_pullback_depth, closes_above, poc_migration_score, value_area_from_b12, volume_oi_proxy,
};

/// Évalue le signal Acceptance Expansion.
#[must_use]
pub fn evaluate(sig_ctx: &SignalContext<'_>) -> SignalOutput {
    let b12 = sig_ctx.module_payload(ModuleId::B12);
    let series = sig_ctx.primary_series();

    let mut score: u8 = 0;
    let mut conditions = Vec::new();

    if let (Some(b12), Some(s)) = (b12, series) {
        let bars = &s.bars;
        if let Some((vah, _)) = value_area_from_b12(b12) {
            let above = closes_above(bars, vah, 3);
            if above >= 3 {
                score = score.saturating_add(30);
                conditions.push(format!("{above} clôtures acceptées au-dessus VAH"));
            }
        }
        let migration = poc_migration_score(bars);
        if migration > 0.008 {
            score = score.saturating_add(25);
            conditions.push("POC migre vers le haut".into());
        }
        let vol_oi = volume_oi_proxy(bars);
        if vol_oi > 1.1 && close_slope_positive(bars) {
            score = score.saturating_add(20);
            conditions.push(format!("volume/OI proxy en hausse ({vol_oi:.2})"));
        }
        let pullback = avg_pullback_depth(bars);
        if pullback < 0.45 {
            score = score.saturating_add(25);
            conditions.push(format!("pullbacks faibles (depth={pullback:.2})"));
        }
        if b12.get("validation").and_then(|v| v.as_str()) == Some("acceptance_above_value") {
            score = score.saturating_add(20);
            conditions.push("B12 confirme acceptation".into());
        }
    }

    let triggered = score >= 65;
    let rationale = if conditions.is_empty() {
        "Acceptance Expansion : pas d'acceptation durable détectée.".into()
    } else {
        format!("Acceptance Expansion : {}.", conditions.join(" ; "))
    };

    SignalOutput {
        kind: SignalKind::AcceptanceExpansion,
        score: score.min(100),
        triggered,
        rationale,
        lineage: B212Lineage::fixture("signal_acceptance_expansion"),
    }
}

fn close_slope_positive(bars: &[crate::types::Bar]) -> bool {
    use crate::modules::common::close_slope;
    close_slope(bars) > 0.0
}