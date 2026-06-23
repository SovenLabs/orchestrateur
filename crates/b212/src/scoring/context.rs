//! Contexte partagé pour le scoring B212.

use crate::modules::ModuleContext;
use crate::types::{ModuleId, ModuleOutput, OhlcvSeries, SignalOutput};

/// Entrée scoring : modules, signaux, session.
#[derive(Debug, Clone)]
pub struct ScoringContext<'a> {
    /// Contexte multi-timeframe.
    pub ctx: &'a ModuleContext,
    /// Modules B1→B12.
    pub modules: &'a [ModuleOutput],
    /// Signaux avancés.
    pub signals: &'a [SignalOutput],
    /// Session de trading.
    pub session: &'a str,
}

impl<'a> ScoringContext<'a> {
    /// Payload JSON d'un module.
    #[must_use]
    pub fn module_payload(&self, id: ModuleId) -> Option<&serde_json::Value> {
        self.modules
            .iter()
            .find(|m| m.module == id)
            .map(|m| &m.payload)
    }

    /// Série OHLCV principale.
    #[must_use]
    pub fn primary_series(&self) -> Option<&OhlcvSeries> {
        self.ctx.primary()
    }

    /// Au moins un signal déclenché.
    #[must_use]
    pub fn any_signal_triggered(&self) -> bool {
        self.signals.iter().any(|s| s.triggered)
    }
}