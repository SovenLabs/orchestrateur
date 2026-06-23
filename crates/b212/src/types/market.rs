use serde::{Deserialize, Serialize};

/// Unité de temps B212.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Timeframe {
    /// 15 minutes (LTF).
    M15,
    /// 1 heure (MTF).
    H1,
    /// 4 heures (HTF).
    H4,
    /// Journalier.
    D1,
}

impl Timeframe {
    /// Libellé court pour fichiers fixture.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::M15 => "15m",
            Self::H1 => "1h",
            Self::H4 => "4h",
            Self::D1 => "1d",
        }
    }
}

/// Scénario de marché associé à une fixture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketScenario {
    /// Tendance directionnelle.
    Trend,
    /// Range / rotation.
    Range,
    /// Compression / squeeze.
    Compression,
}

/// Bougie OHLCV.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bar {
    /// Horodatage ISO-8601 UTC.
    pub ts: String,
    /// Prix d'ouverture.
    pub open: f64,
    /// Plus haut.
    pub high: f64,
    /// Plus bas.
    pub low: f64,
    /// Prix de clôture.
    pub close: f64,
    /// Volume échangé.
    pub volume: f64,
}

/// Série OHLCV normalisée.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OhlcvSeries {
    /// Symbole (ex. `BTCUSDT`).
    pub symbol: String,
    /// Unité de temps.
    pub timeframe: Timeframe,
    /// Scénario pédagogique / test.
    pub scenario: MarketScenario,
    /// Bougies ordonnées chronologiquement.
    pub bars: Vec<Bar>,
}

impl OhlcvSeries {
    /// Tronque la série aux `lookback` dernières bougies.
    #[must_use]
    pub fn tail(&self, lookback: usize) -> Self {
        let start = self.bars.len().saturating_sub(lookback);
        Self {
            symbol: self.symbol.clone(),
            timeframe: self.timeframe,
            scenario: self.scenario,
            bars: self.bars[start..].to_vec(),
        }
    }

    /// Prix de clôture de la dernière bougie.
    #[must_use]
    pub fn last_close(&self) -> Option<f64> {
        self.bars.last().map(|b| b.close)
    }
}