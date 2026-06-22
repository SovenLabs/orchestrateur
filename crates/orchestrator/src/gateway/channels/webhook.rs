use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;

use crate::config::GatewayChannelConfig;
use crate::gateway::error::GatewayError;
use crate::gateway::registry::{Channel, ChannelContext, InboundMessage};

/// Canal webhook HTTP — POST `/v1/channels/webhook`.
#[derive(Debug)]
pub struct WebhookChannel {
    config: GatewayChannelConfig,
}

impl WebhookChannel {
    /// Crée le canal webhook.
    #[must_use]
    pub fn new(config: GatewayChannelConfig) -> Self {
        Self { config }
    }

    /// Vérifie le secret webhook (header `X-Orchestrateur-Webhook-Secret`).
    ///
    /// # Errors
    ///
    /// Retourne [`GatewayError::Unauthorized`] si le secret est invalide.
    pub fn verify_secret(&self, provided: Option<&str>) -> Result<(), GatewayError> {
        if !self.config.enabled {
            return Err(GatewayError::Channel {
                channel: "webhook".into(),
                message: "canal désactivé".into(),
            });
        }
        let expected = std::env::var(&self.config.token_env).map_err(|_| GatewayError::Config(
            format!(
                "variable {} requise pour le webhook",
                self.config.token_env
            ),
        ))?;
        match provided {
            Some(value) if constant_time_eq(value.as_bytes(), expected.as_bytes()) => Ok(()),
            _ => Err(GatewayError::Unauthorized),
        }
    }
}

#[async_trait]
impl Channel for WebhookChannel {
    fn id(&self) -> &str {
        "webhook"
    }

    fn name(&self) -> &str {
        "Webhook HTTP"
    }

    async fn start(&self, _ctx: ChannelContext) -> Result<(), GatewayError> {
        Ok(())
    }

    async fn handle_inbound(&self, message: InboundMessage) -> Result<(), GatewayError> {
        Err(GatewayError::Protocol(format!(
            "webhook inbound traité par HTTP handler: session={}",
            message.session_key
        )))
    }
}

/// Corps JSON du webhook entrant.
#[derive(Debug, Deserialize)]
pub struct WebhookPayload {
    /// Message utilisateur.
    pub message: String,
    /// Clé de session (défaut `webhook`).
    #[serde(default = "default_webhook_session")]
    pub session_key: String,
    /// Identifiant externe pour corrélation.
    pub external_id: Option<String>,
}

fn default_webhook_session() -> String {
    "webhook".into()
}

/// Fabrique le canal webhook.
#[must_use]
pub fn webhook_channel(config: GatewayChannelConfig) -> Arc<dyn Channel> {
    Arc::new(WebhookChannel::new(config))
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in left.iter().zip(right.iter()) {
        diff |= a ^ b;
    }
    diff == 0
}