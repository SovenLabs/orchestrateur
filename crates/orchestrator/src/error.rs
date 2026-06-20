use cortex::{CortexError, EmbeddingError};
use thiserror::Error;

use crate::llm::LlmError;

/// Erreurs de la couche application (orchestrateur).
#[derive(Debug, Error)]
pub enum OrchestratorError {
    /// Erreur propagée depuis le domaine Cortex.
    #[error(transparent)]
    Cortex(#[from] CortexError),

    /// Erreur d'un provider d'embeddings (frontière ports IA).
    #[error(transparent)]
    Embedding(#[from] EmbeddingError),

    /// Erreur d'un provider LLM (frontière ports IA).
    #[error(transparent)]
    Llm(#[from] LlmError),
}

/// Erreurs liées aux Skills.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SkillError {
    /// Skill demandée absente du registre.
    #[error("skill introuvable: {0}")]
    NotFound(String),

    /// Échec lors de l'exécution d'une skill.
    #[error("échec exécution skill: {0}")]
    ExecutionFailed(String),
}