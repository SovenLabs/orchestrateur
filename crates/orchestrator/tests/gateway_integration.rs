//! Tests d'intégration gateway Phase 8 (feature `gateway`).

#![cfg(feature = "gateway")]

use std::sync::Arc;

use cortex::SessionKey;
use orchestrator::gateway::build_gateway_stack;
use orchestrator::testing::MockBundle;
use orchestrator::{OrchestratorConfig, OrchestratorFacade};

fn test_facade() -> Arc<OrchestratorFacade> {
    Arc::new(OrchestratorFacade::new(MockBundle::new().into_deps()))
}

#[test]
fn gateway_config_defaults_to_28789() {
    let cfg = OrchestratorConfig::default();
    assert_eq!(cfg.gateway.port, 28_789);
    assert!(cfg.gateway.enabled);
}

#[tokio::test]
async fn gateway_runner_agent_turn_returns_reply() {
    let facade = test_facade();
    let config = OrchestratorConfig::default();
    std::env::set_var("ORCHESTRATEUR_GATEWAY_TOKEN", "test-token-gateway");

    let (runner, _webhook, _delivery) = build_gateway_stack(facade, &config);
    let reply = runner
        .run_agent_turn(
            "req-1",
            Some(SessionKey::default_chat()),
            None,
            "Bonjour gateway",
            "webchat",
            None,
        )
        .await
        .expect("tour agent");

    assert_eq!(reply, "Bonjour gateway");
}

#[test]
fn gateway_stack_registers_all_catalog_channels() {
    let facade = test_facade();
    let config = OrchestratorConfig::default();
    let (runner, _, _) = build_gateway_stack(facade, &config);
    assert!(runner.registry().channels().len() >= 15);
    assert_eq!(runner.registry().channels().len(), 18);
    assert!(runner.registry().get("webchat").is_some());
    assert!(runner.registry().get("whatsapp").is_some());
}

#[tokio::test]
async fn gateway_runner_verifies_token() {
    let facade = test_facade();
    let config = OrchestratorConfig::default();
    std::env::set_var("ORCHESTRATEUR_GATEWAY_TOKEN", "secret-abc");

    let (runner, _, _) = build_gateway_stack(facade, &config);
    assert!(runner.verify_token("secret-abc").is_ok());
    assert!(runner.verify_token("wrong").is_err());
}