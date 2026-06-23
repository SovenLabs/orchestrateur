//! B1.5 — Market Regime Filter.

use serde_json::json;

use super::common::{atr, close_slope, pct_change};
use super::context::ModuleContext;
use crate::types::{ModuleId, ModuleOutput};

/// Régime de marché B212.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketRegime {
    /// Tendance directionnelle.
    Trend,
    /// Range / rotation.
    Range,
    /// Compression / squeeze.
    Compression,
    /// Expansion post-compression.
    Expansion,
}

impl MarketRegime {
    pub fn label(self) -> &'static str {
        match self {
            Self::Trend => "trend",
            Self::Range => "range",
            Self::Compression => "compression",
            Self::Expansion => "expansion",
        }
    }

    fn strategy(self) -> &'static str {
        match self {
            Self::Trend => "continuation_pullback",
            Self::Range => "fade_extremes",
            Self::Compression => "wait_break_acceptance",
            Self::Expansion => "retest_preferred",
        }
    }
}

/// Détecte le régime actif sur la série principale.
#[must_use]
pub fn analyze(ctx: &ModuleContext) -> ModuleOutput {
    let series = ctx.primary();
    let (regime, slope_pct, vol_ratio, rationale) = match series {
        Some(s) if s.bars.len() >= 8 => {
            let bars = &s.bars;
            let slope = close_slope(bars);
            let slope_pct = pct_change(bars.first().map(|b| b.close).unwrap_or(0.0), bars.last().map(|b| b.close).unwrap_or(0.0));

            let quarter = bars.len() / 4;
            let early_atr = atr(&bars[..quarter.max(2)], quarter.max(2));
            let late_atr = atr(&bars[bars.len() - quarter.max(2)..], quarter.max(2));
            let vol_ratio = if early_atr > f64::EPSILON {
                late_atr / early_atr
            } else {
                1.0
            };

            let regime = if vol_ratio < 0.55 {
                MarketRegime::Compression
            } else if vol_ratio > 1.45 {
                MarketRegime::Expansion
            } else if slope.abs() > 0.025 {
                MarketRegime::Trend
            } else if slope.abs() < 0.008 {
                MarketRegime::Range
            } else if slope > 0.0 {
                MarketRegime::Trend
            } else {
                MarketRegime::Range
            };

            let rationale = format!(
                "B1.5 : pente {slope_pct:.2}%, ratio ATR {vol_ratio:.2} sur {} → régime {}.",
                s.timeframe.label(),
                regime.label()
            );
            (regime, slope_pct, vol_ratio, rationale)
        }
        Some(s) => (
            MarketRegime::Range,
            0.0,
            1.0,
            format!(
                "B1.5 : fenêtre courte ({} barres) — régime range par défaut.",
                s.bars.len()
            ),
        ),
        None => (
            MarketRegime::Range,
            0.0,
            1.0,
            "B1.5 : aucune série — régime range par défaut.".into(),
        ),
    };

    let confidence = match regime {
        MarketRegime::Trend | MarketRegime::Compression => 80,
        MarketRegime::Expansion => 72,
        MarketRegime::Range => 68,
    };

    ModuleOutput {
        module: ModuleId::B1_5,
        summary: format!(
            "Régime {} — stratégie privilégiée : {}.",
            regime.label(),
            regime.strategy()
        ),
        rationale,
        confidence,
        payload: json!({
            "regime": regime.label(),
            "preferred_strategy": regime.strategy(),
            "slope_pct": slope_pct,
            "volatility_ratio": vol_ratio,
        }),
    }
}

/// Extrait le régime depuis un [`ModuleOutput`] B1.5 (tests).
#[must_use]
pub fn regime_from_output(output: &ModuleOutput) -> Option<MarketRegime> {
    let label = output.payload.get("regime")?.as_str()?;
    match label {
        "trend" => Some(MarketRegime::Trend),
        "range" => Some(MarketRegime::Range),
        "compression" => Some(MarketRegime::Compression),
        "expansion" => Some(MarketRegime::Expansion),
        _ => None,
    }
}