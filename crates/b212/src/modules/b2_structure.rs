//! B2 — Structure & timing (BOS, CHoCH, invalidation, FVG).

use serde_json::json;

use super::common::{close_slope, equal_levels, swing_highs, swing_lows};
use super::context::ModuleContext;
use crate::types::{ModuleId, ModuleOutput};

/// Biais structurel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureBias {
    /// Haussier.
    Bull,
    /// Baissier.
    Bear,
    /// Indécis / range.
    Neutral,
}

impl StructureBias {
    fn label(self) -> &'static str {
        match self {
            Self::Bull => "bull",
            Self::Bear => "bear",
            Self::Neutral => "neutral",
        }
    }
}

/// Détecte les FVG bullish récents (gap entre high[i-2] et low[i]).
fn detect_bull_fvgs(bars: &[crate::types::Bar]) -> Vec<f64> {
    let mut gaps = Vec::new();
    for i in 2..bars.len() {
        if bars[i].low > bars[i - 2].high {
            gaps.push((bars[i - 2].high + bars[i].low) / 2.0);
        }
    }
    gaps
}

/// Détecte les FVG bearish.
fn detect_bear_fvgs(bars: &[crate::types::Bar]) -> Vec<f64> {
    let mut gaps = Vec::new();
    for i in 2..bars.len() {
        if bars[i].high < bars[i - 2].low {
            gaps.push((bars[i - 2].low + bars[i].high) / 2.0);
        }
    }
    gaps
}

/// Analyse structurelle B2.
#[must_use]
pub fn analyze(ctx: &ModuleContext) -> ModuleOutput {
    let series = ctx
        .get(crate::types::Timeframe::H1)
        .or_else(|| ctx.get(crate::types::Timeframe::H4))
        .or_else(|| ctx.primary());

    let (
        bias,
        bos,
        choch,
        invalidation,
        liquidity_pools,
        fvgs,
        trade_exists,
        rationale,
    ) = match series {
        Some(s) if s.bars.len() >= 10 => {
            let bars = &s.bars;
            let slope = close_slope(bars);
            let bias = if slope > 0.01 {
                StructureBias::Bull
            } else if slope < -0.01 {
                StructureBias::Bear
            } else {
                StructureBias::Neutral
            };

            let highs = swing_highs(bars, 2);
            let lows = swing_lows(bars, 2);
            let last_close = bars.last().map(|b| b.close).unwrap_or(0.0);
            let last_swing_high = highs.last().map(|(_, v)| *v);
            let last_swing_low = lows.last().map(|(_, v)| *v);

            let bos = match (bias, last_swing_high, last_swing_low) {
                (StructureBias::Bull, Some(sh), _) if last_close > sh => true,
                (StructureBias::Bear, _, Some(sl)) if last_close < sl => true,
                _ => false,
            };

            let choch = match (bias, last_swing_high, last_swing_low) {
                (StructureBias::Bull, _, Some(sl)) if last_close < sl => true,
                (StructureBias::Bear, Some(sh), _) if last_close > sh => true,
                _ => false,
            };

            let invalidation = match bias {
                StructureBias::Bull => last_swing_low,
                StructureBias::Bear => last_swing_high,
                StructureBias::Neutral => last_swing_low.or(last_swing_high),
            };

            let high_levels: Vec<f64> = highs.iter().map(|(_, v)| *v).collect();
            let low_levels: Vec<f64> = lows.iter().map(|(_, v)| *v).collect();
            let eq_highs = equal_levels(&high_levels, 0.15);
            let eq_lows = equal_levels(&low_levels, 0.15);
            let mut liquidity_pools: Vec<f64> = eq_highs;
            liquidity_pools.extend(eq_lows);

            let mut fvgs = detect_bull_fvgs(bars);
            fvgs.extend(detect_bear_fvgs(bars));

            let trade_exists = bos && !choch && invalidation.is_some();
            let fvg_count = fvgs.len();
            let rationale = format!(
                "B2 sur {} : biais {}, BOS={bos}, CHoCH={choch}, invalidation {invalidation:?}, {fvg_count} FVG détectés.",
                s.timeframe.label(),
                bias.label(),
            );

            (
                bias,
                bos,
                choch,
                invalidation,
                liquidity_pools,
                fvgs,
                trade_exists,
                rationale,
            )
        }
        Some(s) => (
            StructureBias::Neutral,
            false,
            false,
            None,
            Vec::new(),
            Vec::new(),
            false,
            format!("B2 : données insuffisantes sur {}.", s.timeframe.label()),
        ),
        None => (
            StructureBias::Neutral,
            false,
            false,
            None,
            Vec::new(),
            Vec::new(),
            false,
            "B2 : aucune série structurelle.".into(),
        ),
    };

    let confidence = if trade_exists {
        78
    } else if bos || !fvgs.is_empty() {
        55
    } else {
        40
    };

    ModuleOutput {
        module: ModuleId::B2,
        summary: format!(
            "Structure {} — trade existe : {} (BOS={bos}, CHoCH={choch}).",
            bias.label(),
            if trade_exists { "oui" } else { "non" }
        ),
        rationale,
        confidence,
        payload: json!({
            "bias": bias.label(),
            "bos": bos,
            "choch": choch,
            "invalidation": invalidation,
            "liquidity_pools": liquidity_pools,
            "fvg_midpoints": fvgs,
            "trade_exists": trade_exists,
        }),
    }
}