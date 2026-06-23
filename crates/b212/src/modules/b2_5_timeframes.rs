//! B2.5 — Timeframe alignment (HTF → MTF → LTF).

use serde_json::json;

use super::b1_5_regime::{analyze as analyze_regime, MarketRegime};
use super::common::close_slope;
use super::context::ModuleContext;
use crate::types::{ModuleId, ModuleOutput, Timeframe};

/// Biais directionnel d'une UT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TfBias {
    Bull,
    Bear,
    Range,
}

impl TfBias {
    fn label(self) -> &'static str {
        match self {
            Self::Bull => "bull",
            Self::Bear => "bear",
            Self::Range => "range",
        }
    }

    fn from_slope(slope: f64) -> Self {
        if slope > 0.012 {
            Self::Bull
        } else if slope < -0.012 {
            Self::Bear
        } else {
            Self::Range
        }
    }
}

fn bias_for_series(ctx: &ModuleContext, tf: Timeframe) -> Option<TfBias> {
    let s = ctx.get(tf)?;
    if s.bars.len() < 4 {
        return None;
    }
    Some(TfBias::from_slope(close_slope(&s.bars)))
}

/// Score d'alignement 0–100.
fn alignment_score(htf: Option<TfBias>, mtf: Option<TfBias>, ltf: Option<TfBias>) -> u8 {
    let biases: Vec<TfBias> = [htf, mtf, ltf].into_iter().flatten().collect();
    if biases.is_empty() {
        return 0;
    }
    if biases.len() == 1 {
        return 50;
    }
    let all_bull = biases.iter().all(|b| *b == TfBias::Bull);
    let all_bear = biases.iter().all(|b| *b == TfBias::Bear);
    if all_bull || all_bear {
        return 90;
    }
    let htf_b = htf.unwrap_or(TfBias::Range);
    let aligned_with_htf = [mtf, ltf]
        .into_iter()
        .flatten()
        .filter(|b| *b == htf_b)
        .count();
    let total = [mtf, ltf].into_iter().flatten().count();
    if total == 0 {
        return 50;
    }
    let ratio = aligned_with_htf as f64 / total as f64;
    (40.0 + ratio * 50.0).round() as u8
}

/// Analyse B2.5 — alignement multi-timeframe.
#[must_use]
pub fn analyze(ctx: &ModuleContext) -> ModuleOutput {
    let htf_bias = bias_for_series(ctx, Timeframe::H4).or_else(|| bias_for_series(ctx, Timeframe::D1));
    let mtf_bias = bias_for_series(ctx, Timeframe::H1);
    let ltf_bias = bias_for_series(ctx, Timeframe::M15);

    let score = alignment_score(htf_bias, mtf_bias, ltf_bias);
    let counter_trend = match (htf_bias, ltf_bias) {
        (Some(TfBias::Bull), Some(TfBias::Bear)) | (Some(TfBias::Bear), Some(TfBias::Bull)) => true,
        _ => false,
    };

    let regime_out = analyze_regime(ctx);
    let regime = super::b1_5_regime::regime_from_output(&regime_out).unwrap_or(MarketRegime::Range);

    let constraints = if counter_trend {
        "taille_réduite, RR_élevé, B12_obligatoire"
    } else if score >= 80 {
        "taille_normale_possible"
    } else if score >= 55 {
        "taille_réduite"
    } else {
        "observation_seule"
    };

    let fmt_bias = |b: Option<TfBias>| b.map(|x| x.label()).unwrap_or("n/a");
    let rationale = format!(
        "B2.5 : HTF={} MTF={} LTF={}, score alignement {score}, contre-tendance={counter_trend}, régime {}.",
        fmt_bias(htf_bias),
        fmt_bias(mtf_bias),
        fmt_bias(ltf_bias),
        regime.label()
    );

    ModuleOutput {
        module: ModuleId::B2_5,
        summary: format!(
            "Alignement {score}/100 — {constraints}."
        ),
        rationale,
        confidence: score,
        payload: json!({
            "htf_bias": htf_bias.map(|b| b.label()),
            "mtf_bias": mtf_bias.map(|b| b.label()),
            "ltf_bias": ltf_bias.map(|b| b.label()),
            "alignment_score": score,
            "counter_trend": counter_trend,
            "constraints": constraints,
        }),
    }
}