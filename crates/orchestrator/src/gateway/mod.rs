//! Gateway WebSocket Phase 8 — protocole typé, canaux, streaming agent.

pub mod channels;
pub mod delivery;
pub mod error;
pub mod protocol;
pub mod registry;
pub mod runtime;
pub mod server;

use std::sync::Arc;

use crate::agent::AgentConfig;
use crate::config::{GatewayChannelConfig, GatewayConfig, OrchestratorConfig};
use crate::facade::OrchestratorFacade;

pub use delivery::{ChannelDelivery, MessageDelivery, OutboundMessage};
pub use error::GatewayError;
pub use protocol::{GatewayClientMessage, GatewayServerMessage};
pub use registry::{Channel, ChannelRegistry, InboundMessage};
pub use runtime::GatewayRunner;
pub use server::{build_router, serve, GatewayState, HealthResponse, WebhookResponse};

pub use channels::ChannelCatalog;

use channels::{
    catalog::CHANNEL_DESCRIPTORS, discord_channel, slack_channel, stub_channel, telegram_channel,
    webchat_channel, webhook_channel, DiscordDelivery, SlackDelivery, TelegramDelivery,
    WebhookChannel,
};

/// Vérifie le token d'un canal (header `X-Orchestrateur-Channel-Token`).
///
/// # Errors
///
/// Retourne [`GatewayError`] si le canal est désactivé, le token absent ou invalide.
pub fn verify_channel_token(
    channel_id: &str,
    config: &GatewayChannelConfig,
    provided: Option<&str>,
) -> Result<(), GatewayError> {
    if !config.enabled {
        return Err(GatewayError::Channel {
            channel: channel_id.into(),
            message: "canal désactivé".into(),
        });
    }
    if config.token_env.is_empty() {
        return Ok(());
    }
    let expected = std::env::var(&config.token_env).map_err(|_| GatewayError::Config(
        format!(
            "variable {} requise pour le canal {channel_id}",
            config.token_env
        ),
    ))?;
    match provided {
        Some(value) if constant_time_eq(value.as_bytes(), expected.as_bytes()) => Ok(()),
        _ => Err(GatewayError::Unauthorized),
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right.iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

/// Profil effectif d'un canal gateway.
#[must_use]
pub fn resolve_channel_config(gateway: &GatewayConfig, id: &str) -> GatewayChannelConfig {
    match id {
        "webhook" => gateway.webhook.clone(),
        "telegram" => gateway.telegram.clone(),
        "discord" => gateway.discord.clone(),
        "slack" => gateway.slack.clone(),
        _ => gateway
            .extra_channels
            .get(id)
            .cloned()
            .unwrap_or_else(|| {
                GatewayChannelConfig::disabled(channels::default_token_env(id).to_string())
            }),
    }
}

/// Assemble le runner gateway, le canal webhook et le livreur multiplexé.
#[must_use]
pub fn build_gateway_stack(
    facade: Arc<OrchestratorFacade>,
    config: &OrchestratorConfig,
) -> (Arc<GatewayRunner>, Arc<WebhookChannel>, Arc<ChannelDelivery>) {
    let gateway_cfg = config.gateway.clone();
    let agent_config = AgentConfig::from_settings(&config.agent);
    let mut registry = ChannelRegistry::new();

    for descriptor in CHANNEL_DESCRIPTORS {
        let channel_cfg = resolve_channel_config(&gateway_cfg, descriptor.id);
        let channel: Arc<dyn Channel> = match descriptor.id {
            "webchat" => webchat_channel(),
            "webhook" => webhook_channel(channel_cfg),
            "telegram" => telegram_channel(channel_cfg),
            "discord" => discord_channel(channel_cfg),
            "slack" => slack_channel(channel_cfg),
            _ => stub_channel(descriptor.id, descriptor.display_name, channel_cfg),
        };
        registry.register(channel);
    }

    let mut delivery = ChannelDelivery::new();
    delivery.register(
        "telegram",
        Arc::new(TelegramDelivery::new(gateway_cfg.telegram.clone())),
    );
    delivery.register("discord", Arc::new(DiscordDelivery::new()));
    delivery.register(
        "slack",
        Arc::new(SlackDelivery::new(gateway_cfg.slack.clone())),
    );

    let delivery = Arc::new(delivery);
    let runner = Arc::new(GatewayRunner::new(
        facade,
        gateway_cfg.clone(),
        registry,
        Arc::clone(&delivery),
        agent_config,
    ));
    let webhook = Arc::new(WebhookChannel::new(gateway_cfg.webhook));
    (runner, webhook, delivery)
}

/// Démarre le gateway complet (canaux + serveur HTTP/WS).
///
/// # Errors
///
/// Propage [`GatewayError`] si le démarrage échoue.
pub async fn run_gateway(
    facade: Arc<OrchestratorFacade>,
    config: &OrchestratorConfig,
) -> Result<(), GatewayError> {
    let (runner, webhook, _delivery) = build_gateway_stack(facade, config);
    runner.start_channels().await?;
    serve(runner, webhook).await
}