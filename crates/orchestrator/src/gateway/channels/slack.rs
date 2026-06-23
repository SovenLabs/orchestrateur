use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use tracing::{debug, info};

use crate::config::GatewayChannelConfig;
use crate::gateway::delivery::{MessageDelivery, OutboundMessage};
use crate::gateway::error::GatewayError;
use crate::gateway::registry::{Channel, ChannelContext, InboundMessage};

/// Enveloppe Events API Slack.
#[derive(Debug, Deserialize)]
pub struct SlackEventEnvelope {
    /// `url_verification` ou `event_callback`.
    pub r#type: String,
    /// Challenge URL verification.
    pub challenge: Option<String>,
    /// Événement imbriqué.
    pub event: Option<SlackInnerEvent>,
}

/// Événement Slack imbriqué.
#[derive(Debug, Deserialize)]
pub struct SlackInnerEvent {
    /// Type d'événement (`message`, …).
    pub r#type: String,
    /// Texte du message.
    pub text: Option<String>,
    /// Utilisateur Slack.
    pub user: Option<String>,
    /// Canal Slack.
    pub channel: Option<String>,
    /// Sous-type (`bot_message` ignoré).
    pub subtype: Option<String>,
}

/// Canal Slack — Events API via `POST /v1/channels/slack/events`.
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

    /// Convertit un événement Slack en message entrant harness.
    #[must_use]
    pub fn envelope_to_inbound(payload: &SlackEventEnvelope) -> Option<InboundMessage> {
        let event = payload.event.as_ref()?;
        if event.r#type != "message" {
            return None;
        }
        if event.subtype.is_some() {
            return None;
        }
        let text = event.text.as_ref()?;
        let user = event.user.as_deref().unwrap_or("unknown");
        let channel = event.channel.as_deref().unwrap_or("unknown");
        Some(InboundMessage {
            channel_id: "slack".into(),
            session_key: format!("slack:{channel}:{user}"),
            text: text.clone(),
            external_id: event.channel.clone(),
        })
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
            info!(
                "slack actif — configurez Events API → POST http://127.0.0.1:28789/v1/channels/slack/events"
            );
        } else {
            debug!("slack désactivé ou token absent");
        }
        Ok(())
    }

    async fn handle_inbound(&self, message: InboundMessage) -> Result<(), GatewayError> {
        debug!(session = %message.session_key, "slack inbound traité");
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