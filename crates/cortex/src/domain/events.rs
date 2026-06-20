use chrono::{DateTime, Utc};

use super::MemoryId;

/// Événement de domaine émis après assimilation réussie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryAssimilated {
    /// Identifiant de la mémoire assimilée.
    pub memory_id: MemoryId,
    /// Horodatage UTC de l'assimilation.
    pub assimilated_at: DateTime<Utc>,
    /// Nombre de backlinks sortants après calcul.
    pub backlink_count: usize,
}

/// Union des événements du domaine Cortex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    /// Une mémoire a été assimilée avec succès.
    MemoryAssimilated(MemoryAssimilated),
}

impl DomainEvent {
    /// Fabrique un événement d'assimilation avec l'horodatage courant.
    pub fn memory_assimilated(memory_id: MemoryId, backlink_count: usize) -> Self {
        Self::MemoryAssimilated(MemoryAssimilated {
            memory_id,
            assimilated_at: Utc::now(),
            backlink_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_assimilation_event() {
        let id = MemoryId::new();
        let event = DomainEvent::memory_assimilated(id, 3);
        match event {
            DomainEvent::MemoryAssimilated(e) => {
                assert_eq!(e.memory_id, id);
                assert_eq!(e.backlink_count, 3);
            }
        }
    }
}