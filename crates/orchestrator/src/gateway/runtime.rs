use std::sync::Arc;

use async_trait::async_trait;
use cortex::SessionKey;
use flume::Sender;
use tracing::info;

use crate::agent::{
    AgentConfig, AgentLoop, AgentStreamEvent, AgentStreamSink, AgentTurnRequest,
};
use crate::config::GatewayConfig;
use crate::facade::OrchestratorFacade;

use super::delivery::{ChannelDelivery, OutboundMessage};
use super::error::GatewayError;
use super::protocol::GatewayServerMessage;
use super::registry::{ChannelContext, ChannelRegistry, InboundHandler, InboundMessage};

/// Runtime gateway — route les messages vers [`AgentLoop`] et émet le streaming.
pub struct GatewayRunner {
    facade: Arc<OrchestratorFacade>,
    config: GatewayConfig,
    registry: ChannelRegistry,
    delivery: Arc<ChannelDelivery>,
    agent_config: AgentConfig,
}

impl GatewayRunner {
    /// Construit le runner avec registre de canaux et livreur.
    #[must_use]
    pub fn new(
        facade: Arc<OrchestratorFacade>,
        config: GatewayConfig,
        registry: ChannelRegistry,
        delivery: Arc<ChannelDelivery>,
        agent_config: AgentConfig,
    ) -> Self {
        Self {
            facade,
            config,
            registry,
            delivery,
            agent_config,
        }
    }

    /// Configuration gateway effective.
    #[must_use]
    pub fn gateway_config(&self) -> &GatewayConfig {
        &self.config
    }

    /// Registre des canaux.
    #[must_use]
    pub fn registry(&self) -> &ChannelRegistry {
        &self.registry
    }

    /// Résout le token gateway depuis l'environnement.
    ///
    /// # Errors
    ///
    /// Retourne [`GatewayError::Config`] si la variable est absente.
    pub fn resolve_token(&self) -> Result<String, GatewayError> {
        std::env::var(&self.config.token_env).map_err(|_| {
            GatewayError::Config(format!(
                "variable {} requise pour le gateway",
                self.config.token_env
            ))
        })
    }

    /// Vérifie un token client.
    pub fn verify_token(&self, provided: &str) -> Result<(), GatewayError> {
        let expected = self.resolve_token()?;
        if constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
            Ok(())
        } else {
            Err(GatewayError::Unauthorized)
        }
    }

    /// Démarre les canaux en arrière-plan.
    ///
    /// # Errors
    ///
    /// Propage [`GatewayError`] si un canal échoue au démarrage.
    pub async fn start_channels(&self) -> Result<(), GatewayError> {
        if !self.config.enabled {
            return Err(GatewayError::Config(
                "gateway désactivé dans orchestrator.toml ([gateway] enabled = false)".into(),
            ));
        }
        let handler = self.inbound_handler();
        let ctx = ChannelContext {
            on_inbound: handler,
        };
        self.registry.start_all(ctx).await?;
        info!("canaux gateway démarrés");
        Ok(())
    }

    /// Exécute un tour agent avec streaming vers un canal WS optionnel.
    ///
    /// # Errors
    ///
    /// Propage [`GatewayError`] si le tour agent échoue.
    pub async fn run_agent_turn(
        &self,
        request_id: &str,
        session_key: SessionKey,
        message: &str,
        channel: &str,
        stream_tx: Option<Sender<GatewayServerMessage>>,
    ) -> Result<String, GatewayError> {
        self.audit_inbound(channel, session_key.as_str(), message);

        let (event_tx, event_rx) = flume::unbounded::<AgentStreamEvent>();
        let stream_sink = AgentStreamSink::from_sender(event_tx);
        let req_id = request_id.to_string();

        if let Some(ws_tx) = stream_tx.clone() {
            let forward_id = req_id.clone();
            tokio::spawn(async move {
                while let Ok(event) = event_rx.recv_async().await {
                    if let Some(msg) = agent_event_to_gateway(&forward_id, event) {
                        if ws_tx.send_async(msg).await.is_err() {
                            break;
                        }
                    }
                }
            });
        } else {
            tokio::spawn(async move {
                while event_rx.recv_async().await.is_ok() {}
            });
        }

        let agent = AgentLoop::new(
            self.facade.deps().clone(),
            self.agent_config.clone(),
            Some(self.facade.skills_registry()),
        );
        let result = agent
            .run_turn_with_stream(
                AgentTurnRequest {
                    session_key,
                    message: message.to_string(),
                },
                stream_sink,
            )
            .await?;

        let outbound = OutboundMessage {
            channel_id: channel.to_string(),
            session_key: result.session_key.to_string(),
            request_id: Some(request_id.to_string()),
            text: result.reply.clone(),
            external_id: None,
        };
        if let Err(err) = self.delivery.deliver(outbound).await {
            tracing::warn!(%err, channel, "livraison sortante échouée");
        }

        Ok(result.reply)
    }

    fn audit_inbound(&self, channel: &str, session_key: &str, message: &str) {
        let details = format!(
            "channel={channel} session={session_key} len={}",
            message.len()
        );
        self.facade
            .deps()
            .security
            .record_security_event("gateway_inbound", &details);
    }

    /// Handler pour les canaux HTTP / polling.
    #[must_use]
    pub fn inbound_handler(&self) -> Arc<dyn InboundHandler> {
        Arc::new(GatewayInboundHandler {
            facade: Arc::clone(&self.facade),
            delivery: Arc::clone(&self.delivery),
            agent_config: self.agent_config.clone(),
        })
    }
}

/// Handler partagé pour les canaux HTTP/polling.
struct GatewayInboundHandler {
    facade: Arc<OrchestratorFacade>,
    delivery: Arc<ChannelDelivery>,
    agent_config: AgentConfig,
}

#[async_trait]
impl InboundHandler for GatewayInboundHandler {
    async fn handle(&self, message: InboundMessage) -> Result<String, GatewayError> {
        let session_key = SessionKey::new(&message.session_key)?;
        self.facade
            .deps()
            .security
            .record_security_event(
                "gateway_inbound",
                &format!(
                    "channel={} session={} len={}",
                    message.channel_id,
                    message.session_key,
                    message.text.len()
                ),
            );

        let agent = AgentLoop::new(
            self.facade.deps().clone(),
            self.agent_config.clone(),
            Some(self.facade.skills_registry()),
        );
        let result = agent
            .run_turn(AgentTurnRequest {
                session_key,
                message: message.text.clone(),
            })
            .await?;

        let outbound = OutboundMessage {
            channel_id: message.channel_id.clone(),
            session_key: result.session_key.to_string(),
            request_id: None,
            text: result.reply.clone(),
            external_id: message.external_id.clone(),
        };
        if let Err(err) = self.delivery.deliver(outbound).await {
            tracing::warn!(%err, channel = %message.channel_id, "livraison sortante échouée");
        }

        Ok(result.reply)
    }
}

fn agent_event_to_gateway(
    request_id: &str,
    event: AgentStreamEvent,
) -> Option<GatewayServerMessage> {
    match event {
        AgentStreamEvent::Delta { content } => {
            Some(GatewayServerMessage::stream_delta(request_id, content))
        }
        AgentStreamEvent::ToolStart { name } => Some(GatewayServerMessage::stream_tool(
            request_id, name, "start", None,
        )),
        AgentStreamEvent::ToolEnd { name, success } => Some(GatewayServerMessage::stream_tool(
            request_id, name, "end", Some(success),
        )),
        AgentStreamEvent::End {
            reply,
            tools_invoked,
        } => Some(GatewayServerMessage::AgentStreamEnd {
            request_id: request_id.to_string(),
            reply,
            tools_invoked,
        }),
        AgentStreamEvent::MessageExpanded { .. }
        | AgentStreamEvent::MessageCompressed { .. }
        | AgentStreamEvent::PreprocessProgress { .. } => None,
    }
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