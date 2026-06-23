//! Cascade Trigger — liquidation / gamma cascade mécanique.

use crate::types::{B212Lineage, ModuleId, SignalKind, SignalOutput};

use super::context::SignalContext;
use super::helpers::{recent_delta_burst, recent_move_pct, volume_oi_proxy};

/// Évalue le signal Cascade Trigger.
#[must_use]
pub fn evaluate(sig_ctx: &SignalContext<'_>) -> SignalOutput {
    let b2 = sig_ctx.module_payload(ModuleId::B2);
    let b12 = sig_ctx.module_payload(ModuleId::B12);
    let series = sig_ctx.primary_series();

    let mut score: u8 = 0;
    let mut conditions = Vec::new();

    if let Some(s) = series {
        let bars = &s.bars;
        let move_pct = recent_move_pct(bars, 4).abs();
        let vol_oi = volume_oi_proxy(bars);
        let delta = recent_delta_burst(bars, 4);

        if b2.and_then(|p| p.get("bos").and_then(|v| v.as_bool())) == Some(true) {
            score = score.saturating_add(25);
            conditions.push("break zone majeure liquidité (BOS)".into());
        }
        if vol_oi > 1.25 {
            score = score.saturating_add(25);
            conditions.push(format!("OI/volume monte avec prix ({vol_oi:.2})"));
        }
        if move_pct > 1.0 && delta.abs() > 0.0 {
            score = score.saturating_add(25);
            conditions.push(format!("delta agressif (burst={delta:.0}, move={move_pct:.2}%)"));
        }
        if move_pct > 2.5 {
            score = score.saturating_add(25);
            conditions.push("mouvement vertical mécanique".into());
        }
        if b12.and_then(|p| p.get("validation").and_then(|v| v.as_str()))
            == Some("acceptance_above_value")
        {
            score = score.saturating_add(15);
            conditions.push("B12 confirme acceptation cascade".into());
        }
    }

    let triggered = score >= 50;
    let rationale = if conditions.is_empty() {
        "Cascade Trigger : pas de cascade détectée.".into()
    } else {
        format!("Cascade Trigger : {}.", conditions.join(" ; "))
    };

    SignalOutput {
        kind: SignalKind::CascadeTrigger,
        score: score.min(100),
        triggered,
        rationale,
        lineage: B212Lineage::fixture("signal_cascade_trigger"),
    }
}