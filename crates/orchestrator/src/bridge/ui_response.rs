//! Interprétation mutualisée des réponses Bridge pour TUI et HUD.

use cortex::DomainEvent;

use super::response::Response;
use super::ui_common::format_health_status;
use crate::security::AuditEvent;
use crate::bridge::types::HubSummary;

/// Action UI suggérée après traitement d'une réponse ou d'un événement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BridgeUiAction {
    /// Aucune action supplémentaire.
    #[default]
    None,
    /// Recharger la liste des mémoires.
    RefreshList,
}

/// Champs extraits d'une réponse [`Response::Health`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthUpdate {
    /// Statut court (`ok`, `degraded`, …).
    pub status: String,
    /// Version orchestrateur.
    pub version: String,
    /// Provider LLM joignable.
    pub llm_available: bool,
    /// Provider embeddings joignable.
    pub embedding_available: bool,
    /// Message barre de statut formaté.
    pub status_message: String,
}

/// Champs extraits d'une réponse [`Response::GraphSummary`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphUpdate {
    /// Nombre de nœuds.
    pub node_count: usize,
    /// Nombre d'arêtes.
    pub edge_count: usize,
    /// Hubs triés.
    pub hubs: Vec<HubSummary>,
    /// Message barre de statut.
    pub status_message: String,
}

/// Champs extraits d'une réponse [`Response::AuditLog`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditUpdate {
    /// Entrées récentes.
    pub entries: Vec<AuditEvent>,
    /// Chaîne BLAKE3 intacte.
    pub chain_intact: bool,
    /// Message barre de statut.
    pub status_message: String,
    /// Alerte utilisateur si la chaîne est rompue.
    pub chain_broken_alert: Option<String>,
}

/// Interprète un [`DomainEvent`] poussé par le fan-out.
#[must_use]
pub fn domain_event_action(event: &DomainEvent) -> (BridgeUiAction, String) {
    match event {
        DomainEvent::MemoryAssimilated(payload) => (
            BridgeUiAction::RefreshList,
            format!("Assimilation réussie ({})", payload.memory_id),
        ),
        DomainEvent::KnowledgeGraphValidated(payload) => (
            BridgeUiAction::None,
            format!(
                "Graphe validé — {} nœuds, {} arêtes",
                payload.node_count, payload.edge_count
            ),
        ),
    }
}

/// Extrait les champs santé d'une réponse Bridge.
#[must_use]
pub fn health_from_response(response: &Response) -> Option<HealthUpdate> {
    match response {
        Response::Health {
            status,
            version,
            llm_available,
            embedding_available,
        } => Some(HealthUpdate {
            status: status.clone(),
            version: version.clone(),
            llm_available: *llm_available,
            embedding_available: *embedding_available,
            status_message: format_health_status(status, *llm_available, *embedding_available),
        }),
        _ => None,
    }
}

/// Extrait les champs graphe d'une réponse Bridge.
#[must_use]
pub fn graph_from_response(response: &Response) -> Option<GraphUpdate> {
    match response {
        Response::GraphSummary {
            node_count,
            edge_count,
            hubs,
        } => Some(GraphUpdate {
            node_count: *node_count,
            edge_count: *edge_count,
            hubs: hubs.clone(),
            status_message: graph_status_message(*node_count, *edge_count),
        }),
        _ => None,
    }
}

/// Extrait les champs audit d'une réponse Bridge.
#[must_use]
pub fn audit_from_response(response: &Response) -> Option<AuditUpdate> {
    match response {
        Response::AuditLog {
            entries,
            chain_intact,
        } => {
            let chain = if *chain_intact { "intacte" } else { "ROMPUE" };
            let status_message =
                format!("Audit : {} entrée(s), chaîne {chain}", entries.len());
            let chain_broken_alert = if *chain_intact {
                None
            } else {
                Some("Chaîne d'audit compromise — vérification requise".into())
            };
            Some(AuditUpdate {
                entries: entries.clone(),
                chain_intact: *chain_intact,
                status_message,
                chain_broken_alert,
            })
        }
        _ => None,
    }
}

/// Message de statut pour un résumé graphe.
#[must_use]
pub fn graph_status_message(node_count: usize, edge_count: usize) -> String {
    format!("Graphe : {node_count} nœuds, {edge_count} arêtes")
}

#[cfg(test)]
mod tests {
    use cortex::MemoryId;

    use super::*;

    #[test]
    fn domain_assimilation_requests_refresh() {
        let event = DomainEvent::memory_assimilated(MemoryId::new(), 2);
        let (action, _) = domain_event_action(&event);
        assert_eq!(action, BridgeUiAction::RefreshList);
    }
}