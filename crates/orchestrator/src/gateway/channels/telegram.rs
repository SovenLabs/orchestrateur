use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use tracing::{debug, warn};

use crate::config::GatewayChannelConfig;
use crate::gateway::delivery::{MessageDelivery, OutboundMessage};
use crate::gateway::error::GatewayError;
use crate::gateway::registry::{Channel, ChannelContext, InboundMessage};

/// Canal Telegram — long-polling `getUpdates` si token configuré.
#[derive(Debug)]
pub struct TelegramChannel {
    config: GatewayChannelConfig,
    http: reqwest::Client,
}

impl TelegramChannel {
    /// Crée le canal Telegram.
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
impl Channel for TelegramChannel {
    fn id(&self) -> &str {
        "telegram"
    }

    fn name(&self) -> &str {
        "Telegram Bot"
    }

    async fn start(&self, ctx: ChannelContext) -> Result<(), GatewayError> {
        let Some(token) = self.token() else {
            debug!("telegram désactivé ou token absent — skip polling");
            return Ok(());
        };

        let http = self.http.clone();
        let handler = Arc::clone(&ctx.on_inbound);
        tokio::spawn(async move {
            let mut offset: i64 = 0;
            loop {
                let url = format!(
                    "https://api.telegram.org/bot{token}/getUpdates?timeout=30&offset={offset}"
                );
                let response = match http.get(&url).send().await {
                    Ok(resp) => resp,
                    Err(err) => {
                        warn!(%err, "telegram getUpdates échoué");
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        continue;
                    }
                };
                let body: TelegramUpdates = match response.json().await {
                    Ok(parsed) => parsed,
                    Err(err) => {
                        warn!(%err, "telegram parse échoué");
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        continue;
                    }
                };
                for update in body.result {
                    if let Some(offset_next) = update.update_id.checked_add(1) {
                        offset = offset_next;
                    }
                    let Some(message) = update.message else {
                        continue;
                    };
                    let Some(text) = message.text else {
                        continue;
                    };
                    let chat_id = message.chat.id.to_string();
                    let session_key = format!("telegram:{chat_id}");
                    let inbound = InboundMessage {
                        channel_id: "telegram".into(),
                        session_key,
                        text,
                        external_id: Some(chat_id),
                    };
                    if let Err(err) = handler.handle(inbound).await {
                        warn!(%err, "telegram inbound échoué");
                    }
                }
            }
        });
        Ok(())
    }

    async fn handle_inbound(&self, _message: InboundMessage) -> Result<(), GatewayError> {
        Ok(())
    }
}

/// Livreur Telegram via `sendMessage`.
#[derive(Debug)]
pub struct TelegramDelivery {
    config: GatewayChannelConfig,
    http: reqwest::Client,
}

impl TelegramDelivery {
    /// Crée le livreur Telegram.
    #[must_use]
    pub fn new(config: GatewayChannelConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl MessageDelivery for TelegramDelivery {
    async fn deliver(&self, message: OutboundMessage) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }
        let token = std::env::var(&self.config.token_env)
            .map_err(|_| format!("variable {} absente", self.config.token_env))?;
        let chat_id = message
            .external_id
            .ok_or_else(|| "external_id requis pour telegram".to_string())?;
        let url = format!("https://api.telegram.org/bot{token}/sendMessage");
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": message.text,
        });
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

#[derive(Debug, Deserialize)]
struct TelegramUpdates {
    result: Vec<TelegramUpdate>,
}

#[derive(Debug, Deserialize)]
struct TelegramUpdate {
    update_id: i64,
    message: Option<TelegramMessage>,
}

#[derive(Debug, Deserialize)]
struct TelegramMessage {
    text: Option<String>,
    chat: TelegramChat,
}

#[derive(Debug, Deserialize)]
struct TelegramChat {
    id: i64,
}

/// Fabrique le canal Telegram.
#[must_use]
pub fn telegram_channel(config: GatewayChannelConfig) -> Arc<dyn Channel> {
    Arc::new(TelegramChannel::new(config))
}