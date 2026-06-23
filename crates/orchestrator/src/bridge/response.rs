use cortex::{DomainEvent, Memory, MemoryId};
use serde::{Deserialize, Serialize};

use crate::draft::StoredDraft;
use crate::security::AuditEvent;

use super::types::{
    AgentMessageSummary, AgentSummary, AppError, B212ProposalSummary, B212SimFillSummary,
    B212WorkflowSummary,
    BridgeSearchHit, DraftSummary, HubIntegritySummary, HubSummary, MarketplaceEntrySummary,
    MemorySummary, SkillSummary, WatcherStatus,
};

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
    /// Réponse du provider LLM (chat libre / agent).
    ChatReply {
        /// Texte généré.
        reply: String,
        /// Outils invoqués pendant le tour (Phase 10).
        #[serde(default)]
        tools_invoked: Vec<String>,
        /// Résumé auto-assimilé en Cortex (si activé).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        auto_assimilated: Option<String>,
        /// Skills auto-exécutées avant le LLM (Phase 14).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        auto_executed_skills: Vec<String>,
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
    /// Catalogue marketplace (Phase 14).
    MarketplaceList {
        /// Version du format catalogue.
        version: u32,
        /// Empreinte BLAKE3 optionnelle.
        #[serde(skip_serializing_if = "Option::is_none")]
        catalog_hash: Option<String>,
        /// Entrées du catalogue.
        entries: Vec<MarketplaceEntrySummary>,
    },
    /// Rapport de vérification d'intégrité du hub (Phase 14).
    HubIntegrityReport {
        /// Détail des manifestes valides / invalides.
        report: HubIntegritySummary,
    },
    /// Statut du watcher de sessions.
    WatcherStatus {
        /// Métriques runtime.
        status: WatcherStatus,
    },
    /// Liste des brouillons en attente.
    DraftList {
        /// Résumés paginés.
        items: Vec<DraftSummary>,
        /// Total.
        total: usize,
    },
    /// Détail complet d'un brouillon.
    DraftDetail {
        /// Enregistrement persisté.
        draft: StoredDraft,
    },
    /// Brouillon publié en mémoire Cortex.
    DraftPublished {
        /// Identifiant du brouillon publié.
        draft_id: String,
        /// Identifiant mémoire créée.
        memory_id: MemoryId,
        /// Titre publié.
        title: String,
    },
    /// Brouillon supprimé.
    DraftDiscarded {
        /// Identifiant supprimé.
        id: String,
    },
    /// Liste des agents persistants.
    AgentList {
        /// Résumés agents.
        items: Vec<AgentSummary>,
    },
    /// Détail agent persistant.
    AgentDetail {
        /// Résumé agent.
        agent: AgentSummary,
    },
    /// Réponse tour agent persistant.
    AgentTurnReply {
        /// Texte de réponse LLM.
        reply: String,
        #[serde(default)]
        /// Outils invoqués pendant le tour.
        tools_invoked: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// ID mémoire auto-assimilée.
        auto_assimilated: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        /// Skills auto-exécutées.
        auto_executed_skills: Vec<String>,
    },
    /// Inbox agent.
    AgentMessages {
        /// Messages reçus.
        items: Vec<AgentMessageSummary>,
    },
    /// Message inter-agent envoyé.
    AgentMessageSent {
        /// Identifiant message.
        message_id: String,
        /// Expéditeur.
        from: String,
        /// Destinataire.
        to: String,
    },
    /// Rapport tâches de fond.
    AgentBackgroundReport {
        /// Messages en attente.
        inbox_count: usize,
        /// Tâches planifiées.
        pending_tasks: usize,
        /// Actions exécutées.
        executed: Vec<String>,
    },
    /// Agent persistant supprimé.
    AgentDeleted {
        /// Identifiant supprimé.
        id: String,
    },
    /// Agents domaine B212 initialisés.
    B212AgentsReady {
        /// Identifiants créés ou existants.
        agent_ids: Vec<String>,
    },
    /// Résultat workflow B212.
    B212Workflow {
        /// Résumé desk.
        result: B212WorkflowSummary,
    },
    /// Propositions B212 en attente.
    B212ProposalList {
        /// Propositions pending.
        items: Vec<B212ProposalSummary>,
    },
    /// Proposition B212 mise à jour (approve/reject).
    B212ProposalUpdated {
        /// Proposition.
        proposal: B212ProposalSummary,
    },
    /// Fill paper B212 exécuté.
    B212SimExecuted {
        /// Proposition mise à jour.
        proposal: B212ProposalSummary,
        /// Fill simulé.
        fill: B212SimFillSummary,
    },
    /// Événement B212 poussé sur le bus (optionnel).
    B212Event(crate::events::B212Event),
}
