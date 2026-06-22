use thiserror::Error;

/// Erreurs du daemon WebSocket local (clients visuels Territoire Graphique).
#[derive(Debug, Error)]
pub enum DaemonError {
    /// Échec réseau (bind, serveur).
    #[error("réseau: {0}")]
    Network(String),
    /// Configuration invalide ou daemon désactivé.
    #[error("configuration: {0}")]
    Config(String),
    /// Token d'authentification absent ou invalide.
    #[error("authentification: {0}")]
    Auth(String),
}