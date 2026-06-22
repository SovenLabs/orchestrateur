use std::sync::Arc;

use async_trait::async_trait;
use tracing::debug;

use crate::config::GatewayChannelConfig;
use crate::gateway::delivery::{MessageDelivery, OutboundMessage};
use crate::gateway::error::GatewayError;
use crate::gateway::registry::{Channel, ChannelContext, InboundMessage};

/// Canal Discord — stub Phase 8 (livraison via webhook URL si configurée).
#[derive(Debug)]
pub struct DiscordChannel {
    config: GatewayChannelConfig,
}

impl DiscordChannel {
    /// Crée le canal Discord.
    #[must_use]
    pub fn new(config: GatewayChannelConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Channel for DiscordChannel {
    fn id(&self) -> &str {
        "discord"
    }

    fn name(&self) -> &str {
        "Discord"
    }

    async fn start(&self, _ctx: ChannelContext) -> Result<(), GatewayError> {
        if self.config.enabled && std::env::var(&self.config.token_env).is_ok() {
            debug!("discord configuré — livraison via webhook URL (DISCORD_WEBHOOK_URL)");
        } else {
            debug!("discord désactivé ou token absent — stub inactif");
        }
        Ok(())
    }

    async fn handle_inbound(&self, message: InboundMessage) -> Result<(), GatewayError> {
        debug!(
            session = %message.session_key,
            "discord inbound stub (intégration Gateway bot Phase 9+)"
        );
        Ok(())
    }
}

/// Livreur Discord via webhook URL (`DISCORD_WEBHOOK_URL`).
#[derive(Debug)]
pub struct DiscordDelivery {
    http: reqwest::Client,
    webhook_env: &'static str,
}

impl DiscordDelivery {
    /// Crée le livreur Discord.
    #[must_use]
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            webhook_env: "DISCORD_WEBHOOK_URL",
        }
    }
}

impl Default for DiscordDelivery {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MessageDelivery for DiscordDelivery {
    async fn deliver(&self, message: OutboundMessage) -> Result<(), String> {
        let url = std::env::var(self.webhook_env)
            .map_err(|_| format!("variable {} absente", self.webhook_env))?;
        let payload = serde_json::json!({ "content": message.text });
        self.http
            .post(url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Fabrique le canal Discord.
#[must_use]
pub fn discord_channel(config: GatewayChannelConfig) -> Arc<dyn Channel> {
    Arc::new(DiscordChannel::new(config))
}