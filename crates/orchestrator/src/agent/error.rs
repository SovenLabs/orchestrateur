use crate::error::OrchestratorError;
use crate::tools::ToolError;
use thiserror::Error;

/// Erreurs de la boucle agent.
#[derive(Debug, Error)]
pub enum AgentError {
    /// Erreur orchestrateur / use case.
    #[error(transparent)]
    Orchestrator(#[from] OrchestratorError),

    /// Erreur outil.
    #[error(transparent)]
    Tool(#[from] ToolError),

    /// Erreur domaine Cortex.
    #[error(transparent)]
    Cortex(#[from] cortex::CortexError),

    /// Erreur LLM.
    #[error(transparent)]
    Llm(#[from] crate::llm::LlmError),

    /// Boucle outil interrompue (max itérations).
    #[error("nombre maximal d'itérations outil atteint ({max})")]
    MaxToolIterations {
        /// Plafond configuré.
        max: usize,
    },

    /// Réponse LLM invalide ou vide.
    #[error("réponse LLM invalide: {0}")]
    InvalidLlmResponse(String),
}