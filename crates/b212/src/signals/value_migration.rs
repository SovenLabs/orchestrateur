//! Value Migration — changement de zone de valeur.

use crate::types::{B212Lineage, ModuleId, SignalKind, SignalOutput};

use super::context::SignalContext;
use super::helpers::{closes_above, poc_from_b12, poc_migration_score, value_area_from_b12};

/// Évalue le signal Value Migration.
#[must_use]
pub fn evaluate(sig_ctx: &SignalContext<'_>) -> SignalOutput {
    let b12 = sig_ctx.module_payload(ModuleId::B12);
    let b2 = sig_ctx.module_payload(ModuleId::B2);
    let series = sig_ctx.primary_series();

    let mut score: u8 = 0;
    let mut conditions = Vec::new();

    if let (Some(b12), Some(s)) = (b12, series) {
        let bars = &s.bars;
        if let Some((vah, _val)) = value_area_from_b12(b12) {
            let above = closes_above(bars, vah, 2);
            if above >= 2 {
                score = score.saturating_add(30);
                conditions.push(format!("{above} clôtures au-dessus VAH {vah:.2}"));
            }
        }
        let migration = poc_migration_score(bars);
        if migration > 0.01 {
            score = score.saturating_add(35);
            conditions.push(format!("POC migre +{migration:.2}%"));
        } else if migration < -0.01 {
            score = score.saturating_add(35);
            conditions.push(format!("POC migre {migration:.2}% (bear)"));
        }
        if let Some(poc) = poc_from_b12(b12) {
            if let Some(last) = bars.last() {
                let dist = (last.close - poc).abs() / poc.max(1.0);
                if dist < 0.005 {
                    score = score.saturating_add(20);
                    conditions.push("prix ancré au nouveau POC".into());
                }
            }
        }
    }

    if let Some(b2) = b2 {
        if b2.get("bos").and_then(|v| v.as_bool()) == Some(true) {
            score = score.saturating_add(15);
            conditions.push("BOS confirme break structure".into());
        }
    }

    let b1_5 = sig_ctx.module_payload(ModuleId::B1_5);
    if b1_5.and_then(|p| p.get("regime")).and_then(|v| v.as_str()) == Some("trend") {
        score = score.saturating_add(10);
        conditions.push("régime trend confirme migration".into());
    }

    let triggered = score >= 50;
    let rationale = if conditions.is_empty() {
        "Value Migration : pas de migration POC/VAH détectée.".into()
    } else {
        format!("Value Migration : {}.", conditions.join(" ; "))
    };

    SignalOutput {
        kind: SignalKind::ValueMigration,
        score: score.min(100),
        triggered,
        rationale,
        lineage: B212Lineage::fixture("signal_value_migration"),
    }
}