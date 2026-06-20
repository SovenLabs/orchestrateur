use thiserror::Error;

/// Erreurs de transport du bridge (canaux, état fermé).
#[derive(Debug, Error)]
pub enum BridgeError {
    /// Le canal de commandes ou de réponses est fermé (HUD ou thread orchestrateur arrêté).
    #[error("canal bridge fermé")]
    ChannelClosed,

    /// Impossible d'obtenir un récepteur d'événements (bridge non initialisé).
    #[error("abonnement événements indisponible")]
    EventSubscriptionUnavailable,

    /// Échec au démarrage du thread système orchestrateur.
    #[error("échec spawn thread orchestrateur: {0}")]
    ThreadSpawn(#[from] std::io::Error),

    /// Échec à la création du runtime Tokio dédié.
    #[error("échec création runtime tokio: {0}")]
    RuntimeBuild(std::io::Error),
}