use async_trait::async_trait;

use crate::domain::CortexError;

/// Port de génération d'embeddings vectoriels.
///
/// Implémentation par défaut prévue : Ollama (`nomic-embed-text`).
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Génère un embedding pour un texte.
    async fn embed(&self, text: &str) -> Result<Vec<f32>, CortexError>;

    /// Génère des embeddings en lot (optimisation réseau).
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, CortexError>;
}