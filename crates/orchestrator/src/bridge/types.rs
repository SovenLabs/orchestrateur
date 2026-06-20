use chrono::{DateTime, Utc};
use cortex::{Memory, MemoryId, SearchHit};
use serde::{Deserialize, Serialize};

use crate::error::OrchestratorError;

/// Vue légère d'une mémoire pour listes virtualisées (HUD / TUI).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemorySummary {
    /// Identifiant unique.
    pub id: MemoryId,
    /// Titre affiché.
    pub title: String,
    /// Tags normalisés (chaînes).
    pub tags: Vec<String>,
    /// Date de création UTC.
    pub created_at: DateTime<Utc>,
    /// Date de dernière modification UTC.
    pub updated_at: DateTime<Utc>,
    /// Nombre de backlinks sortants.
    pub backlink_count: usize,
}

impl MemorySummary {
    /// Construit un résumé depuis une entité [`Memory`] complète.
    #[must_use]
    pub fn from_memory(memory: &Memory) -> Self {
        Self {
            id: memory.id,
            title: memory.title.clone(),
            tags: memory
                .tags
                .iter()
                .map(|tag| tag.as_str().to_string())
                .collect(),
            created_at: memory.created_at,
            updated_at: memory.updated_at,
            backlink_count: memory.backlink_count(),
        }
    }

    /// Indique si le résumé correspond à un filtre textuel (titre ou tags).
    #[must_use]
    pub fn matches_filter(&self, needle: &str) -> bool {
        let needle = needle.to_lowercase();
        self.title.to_lowercase().contains(&needle)
            || self
                .tags
                .iter()
                .any(|tag| tag.to_lowercase().contains(&needle))
    }
}

/// Résultat de recherche exposé au bridge (réutilise le type Cortex).
pub type BridgeSearchHit = SearchHit;

/// Erreur applicative sérialisable pour les réponses bridge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppError {
    /// Catégorie stable (`cortex`, `validation`, `security`, `llm`, …).
    pub kind: String,
    /// Message lisible par l'utilisateur ou le HUD.
    pub message: String,
}

impl AppError {
    /// Construit une erreur applicative depuis une [`OrchestratorError`].
    #[must_use]
    pub fn from_orchestrator(err: &OrchestratorError) -> Self {
        let kind = match err {
            OrchestratorError::Cortex(_) => "cortex",
            OrchestratorError::Embedding(_) => "embedding",
            OrchestratorError::Llm(_) => "llm",
            OrchestratorError::Validation(_) => "validation",
            OrchestratorError::Security(_) => "security",
        };
        Self {
            kind: kind.to_string(),
            message: err.to_string(),
        }
    }
}
