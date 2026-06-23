use serde::{Deserialize, Serialize};

use super::lineage::B212Lineage;

/// Exécution paper persistée (fill simulé).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimFill {
    /// Identifiant unique (`sim-{uuid}`).
    pub id: String,
    /// Proposition source.
    pub proposal_id: String,
    /// Symbole.
    pub symbol: String,
    /// Direction (`long` / `short`).
    pub side: String,
    /// Prix d'entrée simulé (dernière clôture fixture).
    pub entry_price: f64,
    /// Quantité en unités de base.
    pub quantity: f64,
    /// Notionnel USD alloué.
    pub notional_usd: f64,
    /// PnL réalisé USD (0 à l'entrée paper).
    pub realized_pnl_usd: f64,
    /// Horodatage fill ISO-8601 UTC.
    pub fill_ts: String,
    /// Lignée audit.
    pub lineage: B212Lineage,
}