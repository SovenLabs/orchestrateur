use std::sync::Arc;

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, info, warn};

use crate::config::GatewayChannelConfig;
use crate::gateway::delivery::{MessageDelivery, OutboundMessage};
use crate::gateway::error::GatewayError;
use crate::gateway::registry::{Channel, ChannelContext, InboundMessage};

/// Canal Discord — Gateway WebSocket (MESSAGE_CREATE) si token bot configuré.
#[derive(Debug)]
pub struct DiscordChannel {
    config: GatewayChannelConfig,
    http: reqwest::Client,
}

impl DiscordChannel {
    /// Crée le canal Discord.
    #[must_use]
    pub fn new(config: GatewayChannelConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
        }
    }

    fn token(&self) -> Option<String> {
        if !self.config.enabled {
            return None;
        }
        std::env::var(&self.config.token_env).ok()
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

    async fn start(&self, ctx: ChannelContext) -> Result<(), GatewayError> {
        let Some(token) = self.token() else {
            debug!("discord désactivé ou token absent");
            return Ok(());
        };

        let http = self.http.clone();
        let handler = Arc::clone(&ctx.on_inbound);
        tokio::spawn(async move {
            loop {
                if let Err(err) = run_discord_gateway(&http, &token, Arc::clone(&handler)).await {
                    warn!(%err, "discord gateway déconnecté — reconnexion dans 10s");
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                }
            }
        });
        info!("discord gateway démarré (MESSAGE_CREATE)");
        Ok(())
    }

    async fn handle_inbound(&self, message: InboundMessage) -> Result<(), GatewayError> {
        debug!(session = %message.session_key, "discord inbound traité");
        Ok(())
    }
}

async fn run_discord_gateway(
    http: &reqwest::Client,
    token: &str,
    handler: Arc<dyn super::super::registry::InboundHandler>,
) -> Result<(), String> {
    let gateway: GatewayResponse = http
        .get("https://discord.com/api/v10/gateway")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let url = format!("{}?v=10&encoding=json", gateway.url);
    let (ws, _) = connect_async(&url).await.map_err(|e| e.to_string())?;
    let (mut write, mut read) = ws.split();

    let identify = serde_json::json!({
        "op": 2,
        "d": {
            "token": token,
            "intents": 33280_i32, // GUILDS + GUILD_MESSAGES + MESSAGE_CONTENT
            "properties": {
                "os": "orchestrateur",
                "browser": "harness",
                "device": "harness"
            }
        }
    });
    write
        .send(Message::Text(identify.to_string().into()))
        .await
        .map_err(|e| e.to_string())?;

    while let Some(msg) = read.next().await {
        let msg = msg.map_err(|e| e.to_string())?;
        let Message::Text(text) = msg else {
            continue;
        };
        let payload: DiscordDispatch = match serde_json::from_str(&text) {
            Ok(p) => p,
            Err(_) => continue,
        };
        if payload.t.as_deref() != Some("MESSAGE_CREATE") {
            continue;
        }
        let Some(data) = payload.d else {
            continue;
        };
        if data.author.bot.unwrap_or(false) {
            continue;
        }
        let Some(content) = data.content.filter(|c| !c.is_empty()) else {
            continue;
        };
        let author = data.author.id;
        let channel_id = data.channel_id;
        let inbound = InboundMessage {
            channel_id: "discord".into(),
            session_key: format!("discord:{channel_id}:{author}"),
            text: content,
            external_id: Some(channel_id),
        };
        if let Err(err) = handler.handle(inbound).await {
            warn!(%err, "discord inbound échoué");
        }
    }
    Err("discord gateway fermé".into())
}

#[derive(Debug, Deserialize)]
struct GatewayResponse {
    url: String,
}

#[derive(Debug, Deserialize)]
struct DiscordDispatch {
    t: Option<String>,
    d: Option<DiscordMessageCreate>,
}

#[derive(Debug, Deserialize)]
struct DiscordMessageCreate {
    content: Option<String>,
    channel_id: String,
    author: DiscordAuthor,
}

#[derive(Debug, Deserialize)]
struct DiscordAuthor {
    id: String,
    bot: Option<bool>,
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