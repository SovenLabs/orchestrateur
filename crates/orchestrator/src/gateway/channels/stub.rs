use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use tracing::{debug, warn};

use crate::config::GatewayChannelConfig;
use crate::gateway::channels::webhook::WebhookPayload;
use crate::gateway::error::GatewayError;
use crate::gateway::registry::{Channel, ChannelContext, InboundMessage};

/// Canal générique Phase 10 — stub configurable avec polling HTTP Phase 14.
#[derive(Debug)]
pub struct StubChannel {
    id: &'static str,
    display_name: &'static str,
    config: GatewayChannelConfig,
    http: reqwest::Client,
}

impl StubChannel {
    /// Crée un canal stub.
    #[must_use]
    pub fn new(
        id: &'static str,
        display_name: &'static str,
        config: GatewayChannelConfig,
    ) -> Self {
        Self {
            id,
            display_name,
            config,
            http: reqwest::Client::new(),
        }
    }

    fn poll_token(&self) -> Option<String> {
        if self.config.token_env.is_empty() {
            return None;
        }
        std::env::var(&self.config.token_env).ok()
    }
}

#[derive(Debug, Deserialize)]
struct PollMessagesWrapper {
    messages: Vec<WebhookPayload>,
}

/// Parse une réponse de poll : tableau, wrapper `{messages:[]}` ou message unique.
fn parse_poll_payloads(body: &str) -> Vec<WebhookPayload> {
    if let Ok(items) = serde_json::from_str::<Vec<WebhookPayload>>(body) {
        return items;
    }
    if let Ok(wrapper) = serde_json::from_str::<PollMessagesWrapper>(body) {
        return wrapper.messages;
    }
    if let Ok(single) = serde_json::from_str::<WebhookPayload>(body) {
        return vec![single];
    }
    Vec::new()
}

#[async_trait]
impl Channel for StubChannel {
    fn id(&self) -> &str {
        self.id
    }

    fn name(&self) -> &str {
        self.display_name
    }

    async fn start(&self, ctx: ChannelContext) -> Result<(), GatewayError> {
        if !self.config.enabled {
            debug!(channel = self.id, "canal stub inactif");
            return Ok(());
        }

        if let Some(poll_url) = self.config.poll_url.clone() {
            let http = self.http.clone();
            let handler = Arc::clone(&ctx.on_inbound);
            let channel_id = self.id.to_string();
            let interval = std::time::Duration::from_secs(self.config.poll_interval_secs.max(5));
            let token = self.poll_token();
            debug!(
                channel = self.id,
                %poll_url,
                interval_secs = interval.as_secs(),
                "démarrage polling HTTP stub"
            );
            tokio::spawn(async move {
                loop {
                    let mut request = http.get(&poll_url);
                    if let Some(ref value) = token {
                        request = request.header("X-Orchestrateur-Channel-Token", value);
                    }
                    let body = match request.send().await {
                        Ok(resp) => match resp.error_for_status() {
                            Ok(ok) => ok.text().await.unwrap_or_default(),
                            Err(err) => {
                                warn!(channel = %channel_id, %err, "poll stub statut HTTP invalide");
                                tokio::time::sleep(interval).await;
                                continue;
                            }
                        },
                        Err(err) => {
                            warn!(channel = %channel_id, %err, "poll stub échoué");
                            tokio::time::sleep(interval).await;
                            continue;
                        }
                    };
                    if body.trim().is_empty() {
                        tokio::time::sleep(interval).await;
                        continue;
                    }
                    for payload in parse_poll_payloads(&body) {
                        if payload.message.trim().is_empty() {
                            continue;
                        }
                        let session_key = if payload.session_key.is_empty() {
                            format!("{channel_id}:default")
                        } else {
                            payload.session_key.clone()
                        };
                        let inbound = InboundMessage {
                            channel_id: channel_id.clone(),
                            session_key,
                            text: payload.message,
                            external_id: payload.external_id,
                        };
                        if let Err(err) = handler.handle(inbound).await {
                            warn!(channel = %channel_id, %err, "poll stub inbound échoué");
                        }
                    }
                    tokio::time::sleep(interval).await;
                }
            });
            return Ok(());
        }

        if !self.config.token_env.is_empty() {
            if std::env::var(&self.config.token_env).is_ok() {
                debug!(
                    channel = self.id,
                    token_env = %self.config.token_env,
                    "canal stub configuré — inbound HTTP /v1/channels/{id}/inbound"
                );
            } else {
                debug!(
                    channel = self.id,
                    "canal stub activé mais token absent"
                );
            }
        } else {
            debug!(channel = self.id, "canal stub actif sans polling");
        }
        Ok(())
    }

    async fn handle_inbound(&self, message: InboundMessage) -> Result<(), GatewayError> {
        debug!(
            channel = self.id,
            session = %message.session_key,
            "inbound stub traité"
        );
        Ok(())
    }
}

/// Fabrique un canal stub.
#[must_use]
pub fn stub_channel(
    id: &'static str,
    display_name: &'static str,
    config: GatewayChannelConfig,
) -> Arc<dyn Channel> {
    Arc::new(StubChannel::new(id, display_name, config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_poll_payloads_accepts_array_and_wrapper() {
        let array = r#"[{"message":"hi","session_key":"s1"}]"#;
        let items = parse_poll_payloads(array);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].message, "hi");

        let wrapper = r#"{"messages":[{"message":"wrap","session_key":"s2"}]}"#;
        let wrapped = parse_poll_payloads(wrapper);
        assert_eq!(wrapped.len(), 1);
        assert_eq!(wrapped[0].message, "wrap");
    }
}