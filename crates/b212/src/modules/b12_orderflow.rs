//! B12 — Order flow & profiling (validation, ne crée pas de trade).

use serde_json::json;

use super::common::{bar_delta, cumulative_delta};
use super::context::ModuleContext;
use crate::types::{ModuleId, ModuleOutput, Timeframe};

/// POC simplifié : clôture de la bougie au volume maximal.
fn volume_poc(bars: &[crate::types::Bar]) -> Option<f64> {
    bars.iter()
        .max_by(|a, b| {
            a.volume
                .partial_cmp(&b.volume)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|b| b.close)
}

/// VAH / VAL proxy : bornes du quartile de volume autour du POC.
fn value_area(bars: &[crate::types::Bar], poc: f64) -> (f64, f64) {
    let mut weighted: Vec<(f64, f64)> = bars
        .iter()
        .map(|b| (b.close, b.volume))
        .collect();
    weighted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let total_vol: f64 = bars.iter().map(|b| b.volume).sum();
    let target = total_vol * 0.7;
    let poc_idx = weighted
        .iter()
        .position(|(p, _)| (*p - poc).abs() < poc.abs() * 0.001 + 0.01)
        .unwrap_or(weighted.len() / 2);

    let mut acc = weighted[poc_idx].1;
    let mut lo = poc_idx;
    let mut hi = poc_idx;
    while acc < target && (lo > 0 || hi + 1 < weighted.len()) {
        let expand_lo = if lo > 0 {
            Some(weighted[lo - 1].1)
        } else {
            None
        };
        let expand_hi = if hi + 1 < weighted.len() {
            Some(weighted[hi + 1].1)
        } else {
            None
        };
        match (expand_lo, expand_hi) {
            (Some(lv), Some(rv)) if lv >= rv => {
                lo -= 1;
                acc += lv;
            }
            (Some(lv), None) => {
                lo -= 1;
                acc += lv;
            }
            (_, Some(rv)) => {
                hi += 1;
                acc += rv;
            }
            _ => break,
        }
    }
    (weighted[lo].0, weighted[hi].0)
}

/// Analyse B12 — acceptation ou rejet de zone.
#[must_use]
pub fn analyze(ctx: &ModuleContext) -> ModuleOutput {
    let series = ctx
        .get(Timeframe::M15)
        .or_else(|| ctx.get(Timeframe::H1))
        .or_else(|| ctx.primary());

    let (
        poc,
        vah,
        val,
        last_delta,
        cvd,
        absorption,
        validation,
        rationale,
    ) = match series {
        Some(s) if s.bars.len() >= 6 => {
            let bars = &s.bars;
            let poc = volume_poc(bars).unwrap_or(0.0);
            let (val, vah) = value_area(bars, poc);
            let last = bars.last().unwrap();
            let last_delta = bar_delta(last);
            let cvd = cumulative_delta(bars);
            let price_stall = bar_delta(last).signum() != 0.0
                && (last.close - last.open).abs() < (last.high - last.low) * 0.25;
            let absorption = price_stall
                && ((last_delta < 0.0 && last.close >= last.open)
                    || (last_delta > 0.0 && last.close <= last.open));

            let at_value = last.close >= val && last.close <= vah;
            let above_vah = last.close > vah;
            let below_val = last.close < val;

            let validation = if absorption {
                "absorption"
            } else if above_vah && last_delta > 0.0 {
                "acceptance_above_value"
            } else if below_val && last_delta < 0.0 {
                "rejection_below_value"
            } else if at_value {
                "rotation_at_poc"
            } else {
                "inconclusive"
            };

            let rationale = format!(
                "B12 sur {} : POC={poc:.2}, VAH={vah:.2}, VAL={val:.2}, delta={last_delta:.0}, \
                 CVD={cvd:.0}, validation={validation} — B12 valide, ne crée pas.",
                s.timeframe.label()
            );

            (
                poc,
                vah,
                val,
                last_delta,
                cvd,
                absorption,
                validation,
                rationale,
            )
        }
        Some(s) => (
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            false,
            "insufficient_data",
            format!("B12 : fenêtre courte sur {}.", s.timeframe.label()),
        ),
        None => (
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            false,
            "no_data",
            "B12 : aucune série.".into(),
        ),
    };

    let confidence = match validation {
        "absorption" | "acceptance_above_value" | "rejection_below_value" => 75,
        "rotation_at_poc" => 60,
        "inconclusive" => 45,
        _ => 30,
    };

    ModuleOutput {
        module: ModuleId::B12,
        summary: format!(
            "Validation B12 : {validation} — flow confirme ou infirme la zone (ne sauve pas B2)."
        ),
        rationale,
        confidence,
        payload: json!({
            "poc": poc,
            "vah": vah,
            "val": val,
            "last_delta": last_delta,
            "cvd": cvd,
            "absorption": absorption,
            "validation": validation,
            "creates_trade": false,
        }),
    }
}