use async_trait::async_trait;

use crate::domain::{CortexError, MemoryId, Tag};

/// Résultat d'une recherche vectorielle.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchHit {
    pub memory_id: MemoryId,
    pub score: f32,
    pub snippet: Option<String>,
}

/// Filtres optionnels pour la recherche hybride.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SearchFilter {
    pub tags: Vec<Tag>,
    pub min_score: Option<f32>,
    pub limit: Option<usize>,
}

/// Port du vector store local (LanceDB derrière l'infrastructure).
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Insère ou met à jour le vecteur d'une mémoire.
    async fn upsert(&self, memory_id: MemoryId, embedding: &[f32]) -> Result<(), CortexError>;

    /// Recherche sémantique pure.
    async fn semantic_search(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchHit>, CortexError>;

    /// Recherche hybride (sémantique + filtres tags/score).
    async fn hybrid_search(
        &self,
        query_embedding: &[f32],
        filter: &SearchFilter,
    ) -> Result<Vec<SearchHit>, CortexError>;

    /// Supprime l'entrée vectorielle d'une mémoire.
    async fn delete(&self, memory_id: MemoryId) -> Result<(), CortexError>;
}