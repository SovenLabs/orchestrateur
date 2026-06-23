//! Helpers pour scorer les signaux à partir d'OHLCV et payloads modules.

use crate::modules::common::{atr, bar_delta, close_slope, pct_change};
use crate::types::Bar;

/// POC proxy depuis payload B12.
#[must_use]
pub fn poc_from_b12(payload: &serde_json::Value) -> Option<f64> {
    payload.get("poc")?.as_f64()
}

/// VAH / VAL depuis payload B12.
#[must_use]
pub fn value_area_from_b12(payload: &serde_json::Value) -> Option<(f64, f64)> {
    let vah = payload.get("vah")?.as_f64()?;
    let val = payload.get("val")?.as_f64()?;
    Some((vah, val))
}

/// Nombre de clôtures consécutives au-dessus d'un niveau.
#[must_use]
pub fn closes_above(bars: &[Bar], level: f64, min_count: usize) -> usize {
    let mut count = 0;
    for bar in bars.iter().rev() {
        if bar.close > level {
            count += 1;
        } else {
            break;
        }
    }
    if count >= min_count {
        count
    } else {
        0
    }
}

/// POC des premiers vs derniers tiers (migration proxy).
#[must_use]
pub fn poc_migration_score(bars: &[Bar]) -> f64 {
    if bars.len() < 9 {
        return 0.0;
    }
    let third = bars.len() / 3;
    let early_poc = poc_volume_bar(&bars[..third]);
    let late_poc = poc_volume_bar(&bars[bars.len() - third..]);
    if early_poc.abs() < f64::EPSILON {
        return 0.0;
    }
    (late_poc - early_poc) / early_poc
}

fn poc_volume_bar(bars: &[Bar]) -> f64 {
    bars.iter()
        .max_by(|a, b| {
            a.volume
                .partial_cmp(&b.volume)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|b| b.close)
        .unwrap_or(0.0)
}

/// Ratio volume récent / volume ancien (proxy OI build-up).
#[must_use]
pub fn volume_oi_proxy(bars: &[Bar]) -> f64 {
    if bars.len() < 8 {
        return 1.0;
    }
    let half = bars.len() / 2;
    let early_vol: f64 = bars[..half].iter().map(|b| b.volume).sum();
    let late_vol: f64 = bars[half..].iter().map(|b| b.volume).sum();
    let early_avg = early_vol / half as f64;
    let late_avg = late_vol / (bars.len() - half) as f64;
    if early_avg < f64::EPSILON {
        1.0
    } else {
        late_avg / early_avg
    }
}

/// Efficacité prix/volume : variation % / ratio volume.
#[must_use]
pub fn price_volume_divergence(bars: &[Bar]) -> f64 {
    let slope = close_slope(bars);
    let vol_ratio = volume_oi_proxy(bars);
    slope.abs() * 100.0 / vol_ratio.max(0.1)
}

/// Pullback depth proxy : retrace moyen après impulsion locale.
#[must_use]
pub fn avg_pullback_depth(bars: &[Bar]) -> f64 {
    if bars.len() < 5 {
        return 0.0;
    }
    let mut depths = Vec::new();
    for w in bars.windows(3) {
        let impulse = (w[1].high - w[1].low).max(f64::EPSILON);
        let retrace = (w[1].close - w[2].close).abs();
        depths.push(retrace / impulse);
    }
    depths.iter().sum::<f64>() / depths.len() as f64
}

/// Delta agressif net sur les dernières bougies.
#[must_use]
pub fn recent_delta_burst(bars: &[Bar], n: usize) -> f64 {
    bars.iter()
        .rev()
        .take(n)
        .map(bar_delta)
        .sum()
}

/// ATR contraction ratio (compression proxy).
#[must_use]
pub fn atr_compression_ratio(bars: &[Bar]) -> f64 {
    if bars.len() < 8 {
        return 1.0;
    }
    let quarter = bars.len() / 4;
    let early = atr(&bars[..quarter.max(2)], quarter.max(2));
    let late = atr(&bars[bars.len() - quarter.max(2)..], quarter.max(2));
    if early < f64::EPSILON {
        1.0
    } else {
        late / early
    }
}

/// Variation % récente.
#[must_use]
pub fn recent_move_pct(bars: &[Bar], n: usize) -> f64 {
    if bars.len() < n + 1 {
        return 0.0;
    }
    let start = bars[bars.len() - n - 1].close;
    let end = bars.last().map(|b| b.close).unwrap_or(start);
    pct_change(start, end)
}

/// Corps de bougie moyen (raccourcissement impulsions proxy).
#[must_use]
pub fn avg_body_shrink_ratio(bars: &[Bar]) -> f64 {
    if bars.len() < 8 {
        return 1.0;
    }
    let half = bars.len() / 2;
    let early_body: f64 = bars[..half]
        .iter()
        .map(|b| (b.close - b.open).abs())
        .sum::<f64>()
        / half as f64;
    let late_body: f64 = bars[half..]
        .iter()
        .map(|b| (b.close - b.open).abs())
        .sum::<f64>()
        / (bars.len() - half) as f64;
    if early_body < f64::EPSILON {
        1.0
    } else {
        late_body / early_body
    }
}