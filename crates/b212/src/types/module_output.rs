use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Identifiant de module Bible B212.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleId {
    /// B1 — Macro & liquidité globale.
    B1,
    /// B1.5 — Market Regime Filter.
    B1_5,
    /// B2 — Structure & timing.
    B2,
    /// B2.5 — Timeframe alignment.
    B2_5,
    /// B12 — Order flow & profiling.
    B12,
}

/// Sortie standard d'un module (audit + payload structuré).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleOutput {
    /// Module source.
    pub module: ModuleId,
    /// Résumé lisible (6 phrases Bible).
    pub summary: String,
    /// Justification détaillée.
    pub rationale: String,
    /// Score interne 0–100 (module-specific).
    pub confidence: u8,
    /// Données structurées (régime, niveaux, VP, …).
    pub payload: Value,
}