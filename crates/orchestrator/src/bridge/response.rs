use cortex::{DomainEvent, Memory};
use serde::{Deserialize, Serialize};

use super::types::{AppError, BridgeSearchHit, MemorySummary};

/// Réponse produite par le thread orchestrateur vers la couche présentation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Response {
    /// Page de mémoires (liste virtualisée).
    MemoryList {
        /// Éléments de la page courante.
        items: Vec<MemorySummary>,
        /// Total après filtrage (pour pagination).
        total: usize,
    },
    /// Détail complet d'une mémoire.
    MemoryDetail {
        /// Entité domaine complète.
        memory: Memory,
    },
    /// Résultats de recherche vectorielle.
    SearchResults {
        /// Hits ordonnés par pertinence.
        items: Vec<BridgeSearchHit>,
    },
    /// Événement de domaine poussé sur le bus (optionnel via `Response` direct).
    Event(DomainEvent),
    /// Santé du service.
    Health {
        /// Statut court (`ok`, `degraded`, …).
        status: String,
        /// Version du crate orchestrateur.
        version: String,
    },
    /// Erreur métier ou technique sérialisable.
    Error(AppError),
    /// Accusé de réception sans charge utile.
    Success {
        /// Message informatif.
        message: String,
    },
}