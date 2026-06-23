//! Helpers OHLCV partagés entre modules B212.

use crate::types::Bar;

/// Amplitude d'une bougie.
#[must_use]
pub fn bar_range(bar: &Bar) -> f64 {
    bar.high - bar.low
}

/// Amplitude moyenne sur une fenêtre.
#[must_use]
pub fn avg_range(bars: &[Bar]) -> f64 {
    if bars.is_empty() {
        return 0.0;
    }
    bars.iter().map(bar_range).sum::<f64>() / bars.len() as f64
}

/// Pente normalisée des clôtures (régression linéaire simple / premier prix).
#[must_use]
pub fn close_slope(bars: &[Bar]) -> f64 {
    if bars.len() < 2 {
        return 0.0;
    }
    let first = bars.first().map(|b| b.close).unwrap_or(1.0);
    let last = bars.last().map(|b| b.close).unwrap_or(first);
    if first.abs() < f64::EPSILON {
        return 0.0;
    }
    (last - first) / first
}

/// Variation en pourcentage entre deux clôtures.
#[must_use]
pub fn pct_change(from: f64, to: f64) -> f64 {
    if from.abs() < f64::EPSILON {
        return 0.0;
    }
    (to - from) / from * 100.0
}

/// ATR simplifié (moyenne des ranges sur `period` dernières bougies).
#[must_use]
pub fn atr(bars: &[Bar], period: usize) -> f64 {
    if bars.is_empty() {
        return 0.0;
    }
    let start = bars.len().saturating_sub(period);
    avg_range(&bars[start..])
}

/// Détecte les swing highs (sommet local sur `window` barres de chaque côté).
#[must_use]
pub fn swing_highs(bars: &[Bar], window: usize) -> Vec<(usize, f64)> {
    let mut out = Vec::new();
    if bars.len() < window * 2 + 1 {
        return out;
    }
    for i in window..bars.len().saturating_sub(window) {
        let h = bars[i].high;
        let is_high = (0..window).all(|o| h >= bars[i - o - 1].high)
            && (1..=window).all(|o| h > bars[i + o].high);
        if is_high {
            out.push((i, h));
        }
    }
    out
}

/// Détecte les swing lows (creux local).
#[must_use]
pub fn swing_lows(bars: &[Bar], window: usize) -> Vec<(usize, f64)> {
    let mut out = Vec::new();
    if bars.len() < window * 2 + 1 {
        return out;
    }
    for i in window..bars.len().saturating_sub(window) {
        let l = bars[i].low;
        let is_low = (0..window).all(|o| l <= bars[i - o - 1].low)
            && (1..=window).all(|o| l < bars[i + o].low);
        if is_low {
            out.push((i, l));
        }
    }
    out
}

/// Delta signé proxy : `(close - open) * volume`.
#[must_use]
pub fn bar_delta(bar: &Bar) -> f64 {
    (bar.close - bar.open) * bar.volume
}

/// CVD cumulé sur toute la série.
#[must_use]
pub fn cumulative_delta(bars: &[Bar]) -> f64 {
    bars.iter().map(bar_delta).sum()
}

/// Niveaux de prix à fréquence égale (equal highs/lows) — tolérance relative.
#[must_use]
pub fn equal_levels(levels: &[f64], tolerance_pct: f64) -> Vec<f64> {
    if levels.is_empty() {
        return Vec::new();
    }
    let mut clusters: Vec<Vec<f64>> = Vec::new();
    for &level in levels {
        let mut placed = false;
        for cluster in &mut clusters {
            let anchor = cluster[0];
            if anchor.abs() < f64::EPSILON {
                continue;
            }
            if ((level - anchor) / anchor).abs() * 100.0 <= tolerance_pct {
                cluster.push(level);
                placed = true;
                break;
            }
        }
        if !placed {
            clusters.push(vec![level]);
        }
    }
    clusters
        .into_iter()
        .filter(|c| c.len() >= 2)
        .map(|c| c.iter().sum::<f64>() / c.len() as f64)
        .collect()
}