use std::sync::Arc;

use async_trait::async_trait;
use tracing::debug;

use crate::config::GatewayChannelConfig;
use crate::gateway::delivery::{MessageDelivery, OutboundMessage};
use crate::gateway::error::GatewayError;
use crate::gateway::registry::{Channel, ChannelContext, InboundMessage};

/// Canal Slack — stub Phase 8 (livraison `chat.postMessage` si token + channel).
#[derive(Debug)]
pub struct SlackChannel {
    config: GatewayChannelConfig,
}

impl SlackChannel {
    /// Crée le canal Slack.
    #[must_use]
    pub fn new(config: GatewayChannelConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Channel for SlackChannel {
    fn id(&self) -> &str {
        "slack"
    }

    fn name(&self) -> &str {
        "Slack"
    }

    async fn start(&self, _ctx: ChannelContext) -> Result<(), GatewayError> {
        if self.config.enabled && std::env::var(&self.config.token_env).is_ok() {
            debug!("slack configuré — inbound via Events API (Phase 9+)");
        } else {
            debug!("slack désactivé ou token absent — stub inactif");
        }
        Ok(())
    }

    async fn handle_inbound(&self, message: InboundMessage) -> Result<(), GatewayError> {
        debug!(
            session = %message.session_key,
            "slack inbound stub (Events API Phase 9+)"
        );
        Ok(())
    }
}

/// Livreur Slack via `chat.postMessage` (nécessite `SLACK_CHANNEL_ID`).
#[derive(Debug)]
pub struct SlackDelivery {
    config: GatewayChannelConfig,
    http: reqwest::Client,
    channel_env: &'static str,
}

impl SlackDelivery {
    /// Crée le livreur Slack.
    #[must_use]
    pub fn new(config: GatewayChannelConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
            channel_env: "SLACK_CHANNEL_ID",
        }
    }
}

#[async_trait]
impl MessageDelivery for SlackDelivery {
    async fn deliver(&self, message: OutboundMessage) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }
        let token = std::env::var(&self.config.token_env)
            .map_err(|_| format!("variable {} absente", self.config.token_env))?;
        let channel = message
            .external_id
            .or_else(|| std::env::var(self.channel_env).ok())
            .ok_or_else(|| format!("{} ou external_id requis", self.channel_env))?;
        let payload = serde_json::json!({
            "channel": channel,
            "text": message.text,
        });
        self.http
            .post("https://slack.com/api/chat.postMessage")
            .bearer_auth(token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Fabrique le canal Slack.
#[must_use]
pub fn slack_channel(config: GatewayChannelConfig) -> Arc<dyn Channel> {
    Arc::new(SlackChannel::new(config))
}