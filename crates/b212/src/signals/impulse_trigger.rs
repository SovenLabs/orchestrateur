//! Impulse Trigger — protocole 4 conditions (compression, liquidité, reset, break+acceptation).

use crate::types::{B212Lineage, ModuleId, SignalKind, SignalOutput};

use super::context::SignalContext;
use super::helpers::{
    atr_compression_ratio, closes_above, recent_move_pct, value_area_from_b12,
};

/// Évalue le protocole Impulse Trigger (4 conditions).
#[must_use]
pub fn evaluate(sig_ctx: &SignalContext<'_>) -> SignalOutput {
    let b12 = sig_ctx.module_payload(ModuleId::B12);
    let b2 = sig_ctx.module_payload(ModuleId::B2);
    let b1_5 = sig_ctx.module_payload(ModuleId::B1_5);
    let series = sig_ctx.primary_series();

    let mut conditions_met = 0u8;
    let mut conditions = Vec::new();

    if let Some(s) = series {
        let bars = &s.bars;
        let compression = atr_compression_ratio(bars);
        let regime_compression = b1_5
            .and_then(|p| p.get("regime"))
            .and_then(|v| v.as_str())
            == Some("compression")
            || compression < 0.7;
        if regime_compression {
            conditions_met += 1;
            conditions.push(format!("compression (ATR ratio {compression:.2})"));
        }

        let pools = b2
            .and_then(|p| p.get("liquidity_pools"))
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        if pools > 0 {
            conditions_met += 1;
            conditions.push(format!("{pools} pool(s) de liquidité ciblables"));
        }

        let vol_reset = compression < 0.85 && recent_move_pct(bars, 5).abs() < 1.5;
        if vol_reset {
            conditions_met += 1;
            conditions.push("reset dérivés proxy (vol compressée, funding neutre)".into());
        }

        let break_accept = if let Some(b12) = b12 {
            if let Some((vah, _)) = value_area_from_b12(b12) {
                closes_above(bars, vah, 2) >= 2
            } else {
                false
            }
        } else {
            false
        };
        if break_accept {
            conditions_met += 1;
            conditions.push("break + acceptation au-dessus value".into());
        }
    }

    let score = conditions_met.saturating_mul(25);
    let triggered = conditions_met >= 3;
    let rationale = format!(
        "Impulse Trigger : {conditions_met}/4 conditions — {}.",
        if conditions.is_empty() {
            "aucune".into()
        } else {
            conditions.join(" ; ")
        }
    );

    SignalOutput {
        kind: SignalKind::ImpulseTrigger,
        score,
        triggered,
        rationale,
        lineage: B212Lineage::fixture("signal_impulse_trigger"),
    }
}