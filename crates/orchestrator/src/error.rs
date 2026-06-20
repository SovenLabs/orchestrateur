use cortex::CortexError;
use thiserror::Error;

/// Erreurs de la couche application (orchestrateur).
#[derive(Debug, Error)]
pub enum OrchestratorError {
    /// Erreur propagée depuis le domaine Cortex.
    #[error(transparent)]
    Cortex(#[from] CortexError),
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