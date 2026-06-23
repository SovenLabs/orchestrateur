use serde::{Deserialize, Serialize};

use super::lineage::B212Lineage;

/// Statut d'une proposition de trade (Human-in-the-Loop).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    /// En attente validation humaine.
    PendingHuman,
    /// Approuvée par l'opérateur.
    HumanApproved,
    /// Rejetée.
    HumanRejected,
    /// Exécutée en simulation paper.
    SimExecuted,
}

/// Proposition de trade B212 persistée sur disque.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradeProposal {
    /// Identifiant unique.
    pub id: String,
    /// Symbole.
    pub symbol: String,
    /// Session de trading.
    pub session: String,
    /// Direction (`long` / `short` / `observe`).
    pub side: String,
    /// Statut HITL.
    pub status: ProposalStatus,
    /// Trade Location Score /10.
    pub trade_location_score: u8,
    /// Quick Check complet.
    pub quick_check_passed: bool,
    /// Alignment score /10.
    pub alignment_score: u8,
    /// Taille recommandée (`none`, `reduced`, `normal`).
    pub sizing: String,
    /// Résumé 6 phrases Bible.
    pub narrative: String,
    /// Lignée audit.
    pub lineage: B212Lineage,
    /// Horodatage création.
    pub created_at: String,
    /// Motif rejet (optionnel).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reject_reason: Option<String>,
}