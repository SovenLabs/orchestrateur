//! Contexte d'analyse partagé entre modules.

use crate::types::{OhlcvSeries, Timeframe};

/// Contexte multi-timeframe pour l'analyse B212.
#[derive(Debug, Clone)]
pub struct ModuleContext {
    /// Symbole analysé.
    pub symbol: String,
    /// Séries OHLCV disponibles (HTF → LTF).
    pub series: Vec<OhlcvSeries>,
}

impl ModuleContext {
    /// Crée un contexte à partir de séries.
    #[must_use]
    pub fn new(symbol: impl Into<String>, series: Vec<OhlcvSeries>) -> Self {
        Self {
            symbol: symbol.into(),
            series,
        }
    }

    /// Série pour une unité de temps donnée.
    #[must_use]
    pub fn get(&self, tf: Timeframe) -> Option<&OhlcvSeries> {
        self.series.iter().find(|s| s.timeframe == tf)
    }

    /// Série principale : H4 > H1 > M15 > première disponible.
    #[must_use]
    pub fn primary(&self) -> Option<&OhlcvSeries> {
        self.get(Timeframe::H4)
            .or_else(|| self.get(Timeframe::H1))
            .or_else(|| self.get(Timeframe::M15))
            .or_else(|| self.get(Timeframe::D1))
            .or(self.series.first())
    }
}