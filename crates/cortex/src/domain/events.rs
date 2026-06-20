use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::MemoryId;

/// Événement de domaine émis après assimilation réussie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryAssimilated {
    /// Identifiant de la mémoire assimilée.
    pub memory_id: MemoryId,
    /// Horodatage UTC de l'assimilation.
    pub assimilated_at: DateTime<Utc>,
    /// Nombre de backlinks sortants après calcul.
    pub backlink_count: usize,
}

/// Événement émis après validation du graphe de connaissances.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeGraphValidated {
    /// Nombre de nœuds dans le graphe.
    pub node_count: usize,
    /// Nombre total d'arêtes.
    pub edge_count: usize,
}

/// Union des événements du domaine Cortex.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainEvent {
    /// Une mémoire a été assimilée avec succès.
    MemoryAssimilated(MemoryAssimilated),
    /// Le graphe de connaissances a été reconstruit et validé.
    KnowledgeGraphValidated(KnowledgeGraphValidated),
}

impl DomainEvent {
    /// Fabrique un événement d'assimilation avec l'horodatage courant.
    #[must_use]
    pub fn memory_assimilated(memory_id: MemoryId, backlink_count: usize) -> Self {
        Self::MemoryAssimilated(MemoryAssimilated {
            memory_id,
            assimilated_at: Utc::now(),
            backlink_count,
        })
    }

    /// Fabrique un événement de validation du graphe.
    #[must_use]
    pub fn knowledge_graph_validated(node_count: usize, edge_count: usize) -> Self {
        Self::KnowledgeGraphValidated(KnowledgeGraphValidated {
            node_count,
            edge_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_graph_validated_event() {
        let event = DomainEvent::knowledge_graph_validated(3, 5);
        match event {
            DomainEvent::KnowledgeGraphValidated(e) => {
                assert_eq!(e.node_count, 3);
                assert_eq!(e.edge_count, 5);
            }
            _ => panic!("mauvais variant"),
        }
    }

    #[test]
    fn creates_assimilation_event() {
        let id = MemoryId::new();
        let event = DomainEvent::memory_assimilated(id, 3);
        match event {
            DomainEvent::MemoryAssimilated(e) => {
                assert_eq!(e.memory_id, id);
                assert_eq!(e.backlink_count, 3);
            }
            DomainEvent::KnowledgeGraphValidated(_) => panic!("mauvais variant"),
        }
    }
}
