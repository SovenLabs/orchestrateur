//! Registre des canaux messaging et routage des messages entrants.

use std::sync::Arc;

use async_trait::async_trait;

use super::error::GatewayError;

/// Message entrant depuis un canal externe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboundMessage {
    /// Identifiant du canal source.
    pub channel_id: String,
    /// Clé de session agent cible.
    pub session_key: String,
    /// Texte utilisateur.
    pub text: String,
    /// Identifiant externe pour la réponse (chat_id, channel_id Slack, …).
    pub external_id: Option<String>,
}

/// Canal messaging gateway (Telegram, Discord, webhook, …).
#[async_trait]
pub trait Channel: Send + Sync {
    /// Identifiant stable (`telegram`, `discord`, `webhook`, `webchat`, `slack`).
    fn id(&self) -> &str;

    /// Nom affiché.
    fn name(&self) -> &str;

    /// Démarre les tâches de fond du canal (polling, etc.).
    ///
    /// # Errors
    ///
    /// Propage [`GatewayError`] si le démarrage échoue.
    async fn start(&self, ctx: ChannelContext) -> Result<(), GatewayError>;

    /// Traite un message entrant (webhook HTTP ou injection test).
    ///
    /// # Errors
    ///
    /// Propage [`GatewayError`] si le traitement échoue.
    async fn handle_inbound(&self, message: InboundMessage) -> Result<(), GatewayError>;
}

/// Contexte partagé passé aux canaux au démarrage.
#[derive(Clone)]
pub struct ChannelContext {
    /// Callback pour exécuter un tour agent depuis un canal.
    pub on_inbound: Arc<dyn InboundHandler>,
}

/// Handler de messages entrants — implémenté par [`super::runtime::GatewayRunner`].
#[async_trait]
pub trait InboundHandler: Send + Sync {
    /// Traite un message entrant : audit + tour agent + livraison sortante.
    async fn handle(&self, message: InboundMessage) -> Result<String, GatewayError>;
}

/// Registre des canaux actifs.
pub struct ChannelRegistry {
    channels: Vec<Arc<dyn Channel>>,
}

impl ChannelRegistry {
    /// Crée un registre vide.
    #[must_use]
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
        }
    }

    /// Enregistre un canal.
    pub fn register(&mut self, channel: Arc<dyn Channel>) {
        self.channels.push(channel);
    }

    /// Canaux enregistrés.
    #[must_use]
    pub fn channels(&self) -> &[Arc<dyn Channel>] {
        &self.channels
    }

    /// Démarre tous les canaux activés.
    pub async fn start_all(&self, ctx: ChannelContext) -> Result<(), GatewayError> {
        for channel in &self.channels {
            channel.start(ctx.clone()).await?;
        }
        Ok(())
    }

    /// Recherche un canal par identifiant.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<Arc<dyn Channel>> {
        self.channels
            .iter()
            .find(|c| c.id() == id)
            .cloned()
    }
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self::new()
    }
}