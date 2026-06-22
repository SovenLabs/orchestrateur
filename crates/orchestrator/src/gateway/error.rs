use thiserror::Error;

/// Erreur du gateway Phase 8.
#[derive(Debug, Error)]
pub enum GatewayError {
    /// Configuration invalide ou gateway désactivé.
    #[error("configuration gateway: {0}")]
    Config(String),
    /// Authentification refusée.
    #[error("authentification refusée")]
    Unauthorized,
    /// Protocole WebSocket invalide.
    #[error("protocole: {0}")]
    Protocol(String),
    /// Erreur réseau / bind.
    #[error("réseau: {0}")]
    Network(String),
    /// Erreur canal messaging.
    #[error("canal {channel}: {message}")]
    Channel {
        /// Identifiant du canal.
        channel: String,
        /// Détail.
        message: String,
    },
    /// Erreur agent sous-jacente.
    #[error("agent: {0}")]
    Agent(#[from] crate::agent::AgentError),
    /// Erreur Cortex (session key, etc.).
    #[error("cortex: {0}")]
    Cortex(#[from] cortex::CortexError),
    /// Erreur interne.
    #[error("{0}")]
    Internal(String),
}