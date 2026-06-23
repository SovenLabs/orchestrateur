//! False Migration Trap — break sans migration réelle du POC.

use crate::types::{B212Lineage, ModuleId, SignalKind, SignalOutput};

use super::context::SignalContext;
use super::helpers::{
    closes_above, poc_migration_score, recent_delta_burst, value_area_from_b12, volume_oi_proxy,
};

/// Évalue le signal False Migration Trap.
#[must_use]
pub fn evaluate(sig_ctx: &SignalContext<'_>) -> SignalOutput {
    let b12 = sig_ctx.module_payload(ModuleId::B12);
    let series = sig_ctx.primary_series();

    let mut score: u8 = 0;
    let mut conditions = Vec::new();

    if let (Some(b12), Some(s)) = (b12, series) {
        let bars = &s.bars;
        let migration = poc_migration_score(bars).abs();
        let vol_spike = volume_oi_proxy(bars);

        if let Some((vah, val)) = value_area_from_b12(b12) {
            let had_break = closes_above(bars, vah, 1) > 0;
            let reintegrated = bars
                .last()
                .map(|b| b.close < vah && b.close > val)
                .unwrap_or(false);

            if had_break && migration < 0.005 {
                score = score.saturating_add(35);
                conditions.push("break VAH sans migration POC".into());
            }
            if had_break && vol_spike > 1.4 && migration < 0.008 {
                score = score.saturating_add(25);
                conditions.push(format!("volume spike {vol_spike:.2} sur faux break"));
            }
            if reintegrated {
                score = score.saturating_add(30);
                conditions.push("réintégration rapide dans l'ancienne value".into());
            }
        }

        let delta = recent_delta_burst(bars, 3);
        if delta.abs() > 0.0 {
            let last = bars.last().map(|b| b.close).unwrap_or(0.0);
            let prev_high = bars[bars.len().saturating_sub(4)..bars.len().saturating_sub(1)]
                .iter()
                .map(|b| b.high)
                .fold(f64::NEG_INFINITY, f64::max);
            if last < prev_high && delta > 0.0 {
                score = score.saturating_add(20);
                conditions.push("pullback agressif post-break".into());
            }
        }
    }

    let triggered = score >= 55;
    let rationale = if conditions.is_empty() {
        "False Migration Trap : pas de piège détecté.".into()
    } else {
        format!("False Migration Trap : {}.", conditions.join(" ; "))
    };

    SignalOutput {
        kind: SignalKind::FalseMigrationTrap,
        score: score.min(100),
        triggered,
        rationale,
        lineage: B212Lineage::fixture("signal_false_migration"),
    }
}