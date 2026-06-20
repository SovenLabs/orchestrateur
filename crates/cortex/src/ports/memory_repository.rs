use async_trait::async_trait;

use crate::domain::{CortexError, Memory, MemoryId};

/// Port de persistance des mémoires Markdown.
///
/// Implémenté par `infrastructure` (ex: `FileMemoryRepository`).
/// Le domaine ne connaît que ce contrat.
#[async_trait]
pub trait MemoryRepository: Send + Sync {
    /// Persiste ou met à jour une mémoire.
    async fn save(&self, memory: &Memory) -> Result<(), CortexError>;

    /// Récupère une mémoire par identifiant.
    async fn get_by_id(&self, id: MemoryId) -> Result<Memory, CortexError>;

    /// Liste toutes les mémoires (ordre non garanti).
    async fn list(&self) -> Result<Vec<Memory>, CortexError>;

    /// Supprime une mémoire.
    async fn delete(&self, id: MemoryId) -> Result<(), CortexError>;
}
