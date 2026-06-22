use thiserror::Error;

/// Erreur client MCP bas niveau.
#[derive(Debug, Error)]
pub enum McpClientError {
    /// Échec de spawn du processus.
    #[error("spawn {command}: {message}")]
    Spawn {
        /// Commande.
        command: String,
        /// Détail.
        message: String,
    },
    /// Erreur I/O stdio.
    #[error("IO MCP: {0}")]
    Io(String),
    /// JSON invalide.
    #[error("JSON MCP: {0}")]
    Json(String),
    /// Erreur JSON-RPC.
    #[error("RPC {code}: {message}")]
    Rpc {
        /// Code erreur.
        code: i64,
        /// Message.
        message: String,
    },
    /// Timeout.
    #[error("timeout MCP")]
    Timeout,
}