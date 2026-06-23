use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::lineage::B212Lineage;

/// Type d'événement journal B212 (JSONL append-only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JournalEventKind {
    /// Analyse setup complétée.
    SetupAnalyzed,
    /// Proposition bloquée par règles cardinales.
    CardinalBlocked,
    /// Proposition trade créée (HITL).
    ProposalCreated,
    /// Proposition approuvée par l'opérateur.
    ProposalApproved,
    /// Proposition rejetée.
    ProposalRejected,
    /// Proposition exécutée en simulation paper.
    ProposalSimExecuted,
}

/// Entrée journal B212 (une ligne JSONL).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Horodatage UTC ISO-8601.
    pub timestamp: String,
    /// Type d'événement.
    pub event_kind: JournalEventKind,
    /// Symbole.
    pub symbol: String,
    /// Session.
    pub session: String,
    /// Identifiant proposition (si applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposal_id: Option<String>,
    /// Payload structuré (scores, violations, statut, …).
    pub details: Value,
    /// Lignée audit.
    pub lineage: B212Lineage,
}