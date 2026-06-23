//! Contexte partagé pour l'évaluation des signaux.

use crate::modules::ModuleContext;
use crate::types::{ModuleId, ModuleOutput, OhlcvSeries};

/// Entrée signaux : contexte marché + sorties modules B1→B12.
#[derive(Debug, Clone)]
pub struct SignalContext<'a> {
    /// Contexte multi-timeframe.
    pub ctx: &'a ModuleContext,
    /// Modules déjà calculés.
    pub modules: &'a [ModuleOutput],
}

impl<'a> SignalContext<'a> {
    /// Payload JSON d'un module par identifiant.
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
}