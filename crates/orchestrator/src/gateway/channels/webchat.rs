use std::sync::Arc;

use async_trait::async_trait;

use crate::gateway::error::GatewayError;
use crate::gateway::registry::{Channel, ChannelContext, InboundMessage};

/// Canal WebChat — messages via WebSocket `agent.send` (pas de polling).
#[derive(Debug, Default)]
pub struct WebChatChannel;

#[async_trait]
impl Channel for WebChatChannel {
    fn id(&self) -> &str {
        "webchat"
    }

    fn name(&self) -> &str {
        "WebChat (WebSocket)"
    }

    async fn start(&self, _ctx: ChannelContext) -> Result<(), GatewayError> {
        Ok(())
    }

    async fn handle_inbound(&self, _message: InboundMessage) -> Result<(), GatewayError> {
        Err(GatewayError::Protocol(
            "webchat utilise le protocole WebSocket agent.send".into(),
        ))
    }
}

/// Fabrique le canal WebChat.
#[must_use]
pub fn webchat_channel() -> Arc<dyn Channel> {
    Arc::new(WebChatChannel)
}