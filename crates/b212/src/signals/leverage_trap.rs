//! Leverage Build-Up Trap — anti-pattern long tardif.

use crate::types::{B212Lineage, ModuleId, SignalKind, SignalOutput};

use super::context::SignalContext;
use super::helpers::{
    avg_body_shrink_ratio, avg_pullback_depth, price_volume_divergence, volume_oi_proxy,
};

/// Évalue le signal Leverage Build-Up Trap.
#[must_use]
pub fn evaluate(sig_ctx: &SignalContext<'_>) -> SignalOutput {
    let b12 = sig_ctx.module_payload(ModuleId::B12);
    let series = sig_ctx.primary_series();

    let mut score: u8 = 0;
    let mut conditions = Vec::new();

    if let Some(s) = series {
        let bars = &s.bars;
        let vol_oi = volume_oi_proxy(bars);
        let divergence = price_volume_divergence(bars);
        let body_shrink = avg_body_shrink_ratio(bars);
        let pullback = avg_pullback_depth(bars);

        if vol_oi > 1.35 {
            use crate::modules::common::close_slope;
            let slope = close_slope(bars);
            if vol_oi > slope.abs() * 50.0 + 1.0 {
                score = score.saturating_add(30);
                conditions.push(format!("OI monte plus vite que le prix ({vol_oi:.2})"));
            }
        }
        if body_shrink < 0.75 {
            score = score.saturating_add(25);
            conditions.push(format!("impulsions qui raccourcissent ({body_shrink:.2})"));
        }
        if pullback > 0.55 {
            score = score.saturating_add(25);
            conditions.push(format!("pullbacks qui s'approfondissent ({pullback:.2})"));
        }
        if divergence < 0.8 {
            score = score.saturating_add(20);
            conditions.push(format!("delta divergent/inefficace ({divergence:.2})"));
        }
        if let Some(b12) = b12 {
            if b12.get("absorption").and_then(|v| v.as_bool()) == Some(true) {
                score = score.saturating_add(15);
                conditions.push("absorption au sommet (support fragile)".into());
            }
        }
    }

    let triggered = score >= 55;
    let rationale = if conditions.is_empty() {
        "Leverage Trap : pas de build-up excessif détecté.".into()
    } else {
        format!("Leverage Trap : {}.", conditions.join(" ; "))
    };

    SignalOutput {
        kind: SignalKind::LeverageTrap,
        score: score.min(100),
        triggered,
        rationale,
        lineage: B212Lineage::fixture("signal_leverage_trap"),
    }
}