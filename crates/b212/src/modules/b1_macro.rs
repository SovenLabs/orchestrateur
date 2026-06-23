//! B1 — Macro & liquidité globale (contexte, jamais déclencheur).

use serde_json::json;

use super::common::{atr, close_slope, pct_change};
use super::context::ModuleContext;
use crate::types::{ModuleId, ModuleOutput};

/// Climat de liquidité B1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroClimate {
    /// Expansion / biais favorable au risque.
    Favorable,
    /// Neutre — sélectivité normale.
    Neutral,
    /// Contraction / stress — prudence.
    Hostile,
}

impl MacroClimate {
    fn label(self) -> &'static str {
        match self {
            Self::Favorable => "favorable",
            Self::Neutral => "neutre",
            Self::Hostile => "hostile",
        }
    }
}

/// Analyse B1 : ajuste agressivité, ne déclenche jamais d'entrée.
#[must_use]
pub fn analyze(ctx: &ModuleContext) -> ModuleOutput {
    let series = ctx.primary();
    let (climate, slope_pct, vol_ratio, rationale) = match series {
        Some(s) if s.bars.len() >= 6 => {
            let bars = &s.bars;
            let half = bars.len() / 2;
            let early = &bars[..half];
            let late = &bars[half..];
            let slope = close_slope(bars);
            let slope_pct = pct_change(
                early.last().map(|b| b.close).unwrap_or(0.0),
                late.last().map(|b| b.close).unwrap_or(0.0),
            );
            let early_atr = atr(early, early.len().max(1));
            let late_atr = atr(late, late.len().max(1));
            let vol_ratio = if early_atr > f64::EPSILON {
                late_atr / early_atr
            } else {
                1.0
            };

            let climate = if vol_ratio < 0.65 {
                MacroClimate::Hostile
            } else if slope > 0.015 && vol_ratio >= 0.85 {
                MacroClimate::Favorable
            } else if slope < -0.02 && vol_ratio < 0.8 {
                MacroClimate::Hostile
            } else {
                MacroClimate::Neutral
            };

            let rationale = format!(
                "B1 lit le climat via pente {slope_pct:.2}% et ratio volatilité {vol_ratio:.2} \
                 sur {} — le macro contextualise, il ne déclenche pas.",
                s.timeframe.label()
            );
            (climate, slope_pct, vol_ratio, rationale)
        }
        Some(s) => (
            MacroClimate::Neutral,
            0.0,
            1.0,
            format!(
                "B1 : données insuffisantes sur {} ({} barres) — climat neutre par défaut.",
                s.timeframe.label(),
                s.bars.len()
            ),
        ),
        None => (
            MacroClimate::Neutral,
            0.0,
            1.0,
            "B1 : aucune série disponible — climat neutre.".into(),
        ),
    };

    let aggressiveness = match climate {
        MacroClimate::Favorable => "normal",
        MacroClimate::Neutral => "selective",
        MacroClimate::Hostile => "reduced",
    };

    let confidence = match climate {
        MacroClimate::Favorable => 75,
        MacroClimate::Neutral => 60,
        MacroClimate::Hostile => 70,
    };

    ModuleOutput {
        module: ModuleId::B1,
        summary: format!(
            "Climat macro {} — agressivité {aggressiveness} (B1 ne déclenche pas).",
            climate.label()
        ),
        rationale,
        confidence,
        payload: json!({
            "climate": climate.label(),
            "aggressiveness": aggressiveness,
            "slope_pct": slope_pct,
            "volatility_ratio": vol_ratio,
            "triggers_entry": false,
        }),
    }
}