//! Smoke harness intégré.

use std::path::Path;

use crate::bridge::{execute_command, Command, Response};
use crate::config::OrchestratorConfig;
use crate::harness::client::probe_client;
use crate::harness::error::HarnessError;
use crate::harness::probe::probe_harness_services;
use crate::harness::types::HarnessSmokeOptions;
use crate::OrchestratorFacade;

/// Étapes smoke réussies.
#[derive(Debug, Clone)]
pub struct SmokeResult {
    /// Noms des étapes OK.
    pub steps: Vec<String>,
}

/// Enchaîne health, graph, watcher, drafts, memories, chat optionnel.
pub async fn run_smoke(
    facade: &OrchestratorFacade,
    workspace: &Path,
    opts: &HarnessSmokeOptions,
) -> Result<SmokeResult, HarnessError> {
    if !opts.skip_gateway {
        let config = OrchestratorConfig::load_workspace(workspace)?;
        if config.gateway.enabled {
            let probe = probe_harness_services(&config, &probe_client()).await;
            if probe.gateway != "alive" && probe.gateway != "skipped" {
                return Err(HarnessError::GatewayDown {
                    url: probe.gateway_url,
                });
            }
        }
    }

    let mut steps: Vec<(&str, Command)> = vec![
        ("health", Command::HealthCheck),
        ("graph", Command::Graph),
        ("watcher", Command::WatcherStatus),
        ("drafts", Command::ListDrafts),
        (
            "memories",
            Command::List {
                filter: None,
                offset: 0,
                limit: 3,
            },
        ),
    ];
    if !opts.skip_chat {
        steps.push((
            "chat",
            Command::Chat {
                message: "smoke harness — ping agent".into(),
            },
        ));
    }

    let mut completed = Vec::new();
    for (name, cmd) in steps {
        let response = execute_command(facade, cmd).await;
        if matches!(response, Response::Error(_)) {
            return Err(HarnessError::SmokeFailed {
                step: name.into(),
            });
        }
        completed.push(name.to_string());
    }

    Ok(SmokeResult { steps: completed })
}