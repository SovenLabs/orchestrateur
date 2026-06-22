//! Tests inbound HTTP canaux stub Phase 12 (feature `gateway`).

#![cfg(feature = "gateway")]

use orchestrator::config::GatewayChannelConfig;
use orchestrator::gateway::{build_gateway_stack, resolve_channel_config, verify_channel_token};
use orchestrator::testing::MockBundle;
use orchestrator::OrchestratorConfig;
use std::sync::Arc;

#[test]
fn stub_channels_are_registered_in_gateway_stack() {
    let facade = Arc::new(orchestrator::OrchestratorFacade::new(
        MockBundle::new().into_deps(),
    ));
    let config = OrchestratorConfig::default();
    let (runner, _, _) = build_gateway_stack(facade, &config);
    for id in ["whatsapp", "matrix", "signal", "nostr"] {
        assert!(
            runner.registry().get(id).is_some(),
            "canal stub manquant: {id}"
        );
    }
}

#[test]
fn verify_channel_token_matches_env() {
    std::env::set_var("TEST_CHANNEL_TOKEN", "channel-secret");
    let cfg = GatewayChannelConfig {
        enabled: true,
        token_env: "TEST_CHANNEL_TOKEN".into(),
        poll_url: None,
        poll_interval_secs: 30,
    };
    assert!(verify_channel_token("whatsapp", &cfg, Some("channel-secret")).is_ok());
    assert!(verify_channel_token("whatsapp", &cfg, Some("wrong")).is_err());
}

#[test]
fn resolve_stub_channel_uses_extra_channels_or_defaults() {
    let mut config = OrchestratorConfig::default();
    config.gateway.extra_channels.insert(
        "whatsapp".into(),
        GatewayChannelConfig {
            enabled: true,
            token_env: "CUSTOM_WHATSAPP".into(),
            poll_url: Some("http://127.0.0.1:9999/poll".into()),
            poll_interval_secs: 15,
        },
    );
    let cfg = resolve_channel_config(&config.gateway, "whatsapp");
    assert_eq!(cfg.token_env, "CUSTOM_WHATSAPP");
    assert_eq!(
        cfg.poll_url.as_deref(),
        Some("http://127.0.0.1:9999/poll")
    );
    assert_eq!(cfg.poll_interval_secs, 15);
    let matrix = resolve_channel_config(&config.gateway, "matrix");
    assert_eq!(matrix.token_env, "MATRIX_ACCESS_TOKEN");
}