use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::lineage::B212Lineage;
use super::module_output::ModuleOutput;
use super::signal::SignalOutput;
use super::Timeframe;

/// Contexte d'analyse d'un setup (entrée pipeline).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetupContext {
    /// Symbole analysé.
    pub symbol: String,
    /// Session (ex. `london`, `ny`).
    pub session: String,
    /// Unités de temps demandées (HTF → LTF).
    pub timeframes: Vec<Timeframe>,
    /// Nombre de bougies par timeframe.
    pub lookback: usize,
}

/// Analyse complète d'un setup (modules + scores — PR-2+ remplissent les champs).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct B212SetupAnalysis {
    /// Symbole.
    pub symbol: String,
    /// Session.
    pub session: String,
    /// Sorties modules (B1 → B12).
    #[serde(default)]
    pub modules: Vec<ModuleOutput>,
    /// Signaux avancés (PR-3).
    #[serde(default)]
    pub signals: Vec<SignalOutput>,
    /// Bundle scoring (PR-4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scores: Option<Value>,
    /// Lignée audit.
    pub lineage: B212Lineage,
}