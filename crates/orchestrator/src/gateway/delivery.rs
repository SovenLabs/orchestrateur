//! Livraison des messages sortants vers les canaux messaging.

use std::sync::Arc;

use async_trait::async_trait;
use tracing::debug;

/// Message sortant à livrer sur un canal externe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundMessage {
    /// Identifiant du canal cible.
    pub channel_id: String,
    /// Clé de session agent.
    pub session_key: String,
    /// Identifiant de corrélation (optionnel).
    pub request_id: Option<String>,
    /// Contenu textuel à livrer.
    pub text: String,
    /// Identifiant de conversation externe (chat_id Telegram, etc.).
    pub external_id: Option<String>,
}

/// Livreur de messages sortants vers les canaux.
#[async_trait]
pub trait MessageDelivery: Send + Sync {
    /// Livre un message sortant.
    async fn deliver(&self, message: OutboundMessage) -> Result<(), String>;
}

/// Livreur no-op (webchat — réponses via WebSocket uniquement).
#[derive(Debug, Default)]
pub struct NoopDelivery;

#[async_trait]
impl MessageDelivery for NoopDelivery {
    async fn deliver(&self, message: OutboundMessage) -> Result<(), String> {
        debug!(
            channel = %message.channel_id,
            session = %message.session_key,
            "livraison noop (réponse déjà streamée)"
        );
        Ok(())
    }
}

/// Livreur multiplexé vers les canaux enregistrés.
#[derive(Default)]
pub struct ChannelDelivery {
    handlers: std::collections::HashMap<String, Arc<dyn MessageDelivery>>,
}

impl ChannelDelivery {
    /// Crée un livreur vide.
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: std::collections::HashMap::new(),
        }
    }

    /// Enregistre un handler pour un canal.
    pub fn register(&mut self, channel_id: impl Into<String>, delivery: Arc<dyn MessageDelivery>) {
        self.handlers.insert(channel_id.into(), delivery);
    }

    /// Livre via le handler du canal (noop si absent).
    pub async fn deliver(&self, message: OutboundMessage) -> Result<(), String> {
        if let Some(handler) = self.handlers.get(&message.channel_id) {
            handler.deliver(message).await
        } else {
            NoopDelivery.deliver(message).await
        }
    }
}