use serde::{Deserialize, Serialize};

use super::lineage::B212Lineage;

/// Identifiant canonique d'un signal avancé B212.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalKind {
    /// Migration de la value area.
    ValueMigration,
    /// Acceptation de la nouvelle value.
    AcceptanceExpansion,
    /// Piège de fausse migration.
    FalseMigrationTrap,
    /// Protocole impulsion (4 conditions).
    ImpulseTrigger,
    /// Cascade liquidation / gamma.
    CascadeTrigger,
    /// Piège de levier excessif.
    LeverageTrap,
}

impl SignalKind {
    /// Libellé wire JSON.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ValueMigration => "value_migration",
            Self::AcceptanceExpansion => "acceptance_expansion",
            Self::FalseMigrationTrap => "false_migration_trap",
            Self::ImpulseTrigger => "impulse_trigger",
            Self::CascadeTrigger => "cascade_trigger",
            Self::LeverageTrap => "leverage_trap",
        }
    }
}

/// Sortie d'un signal avancé (audit + score).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalOutput {
    /// Type de signal.
    pub kind: SignalKind,
    /// Score de confiance 0–100.
    pub score: u8,
    /// Signal déclenché (seuil module atteint).
    pub triggered: bool,
    /// Justification lisible.
    pub rationale: String,
    /// Lignée audit.
    pub lineage: B212Lineage,
}