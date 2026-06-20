use cortex::{DomainEvent, Memory, MemoryId};
use serde::{Deserialize, Serialize};

use crate::security::AuditEvent;

use super::types::{AppError, BridgeSearchHit, HubSummary, MemorySummary, SkillSummary};

/// Réponse produite par le thread orchestrateur vers la couche présentation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "response", content = "payload")]
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
        /// Provider LLM joignable.
        llm_available: bool,
        /// Provider d'embeddings joignable (recherche sémantique).
        embedding_available: bool,
    },
    /// Erreur métier ou technique sérialisable.
    Error(AppError),
    /// Accusé de réception sans charge utile.
    Success {
        /// Message informatif.
        message: String,
    },
    /// Assimilation réussie (détail complet optionnel via événement domaine).
    Assimilated {
        /// Identifiant de la mémoire créée.
        memory_id: MemoryId,
        /// Titre de la mémoire assimilée.
        title: String,
    },
    /// Résumé du graphe de connaissances.
    GraphSummary {
        /// Nombre de nœuds.
        node_count: usize,
        /// Nombre d'arêtes.
        edge_count: usize,
        /// Hubs triés par backlinks entrants.
        hubs: Vec<HubSummary>,
    },
    /// Entrées récentes du journal d'audit.
    AuditLog {
        /// Entrées lues (ordre chronologique).
        entries: Vec<AuditEvent>,
        /// Chaîne BLAKE3 intacte sur le fichier complet.
        chain_intact: bool,
    },
    /// Réponse du provider LLM (chat libre).
    ChatReply {
        /// Texte généré.
        reply: String,
    },
    /// Catalogue des skills disponibles.
    SkillList {
        /// Paires nom / description.
        skills: Vec<SkillSummary>,
    },
    /// Résultat d'exécution d'une skill.
    SkillResult {
        /// Message ou payload textuel.
        message: String,
    },
}
