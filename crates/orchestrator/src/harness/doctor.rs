//! Diagnostic harness structuré.

use std::path::Path;

use crate::bridge::{execute_command, Command, Response};
use crate::config::OrchestratorConfig;
use crate::harness::client::probe_client;
use crate::harness::daemon::probe_service_url;
use crate::harness::error::HarnessError;
use crate::harness::types::{CheckStatus, DoctorCheck, DoctorReport};
use crate::health::probe_services;
use crate::security::assert_llm_egress_allowed;
use crate::OrchestratorFacade;

/// Exécute le diagnostic complet et retourne un rapport structuré.
pub async fn run_doctor(
    facade: &OrchestratorFacade,
    workspace: &Path,
) -> Result<DoctorReport, HarnessError> {
    let config = OrchestratorConfig::load_workspace(workspace)?;
    let mut checks = Vec::new();
    let mut issues = 0usize;

    for (label, cmd) in [
        ("health bridge", Command::HealthCheck),
        ("graphe", Command::Graph),
        ("watcher", Command::WatcherStatus),
        ("drafts", Command::ListDrafts),
    ] {
        match bridge_ok(facade, cmd).await {
            Ok(()) => checks.push(DoctorCheck {
                label: label.into(),
                status: CheckStatus::Ok,
                detail: None,
            }),
            Err(detail) => {
                issues += 1;
                checks.push(DoctorCheck {
                    label: label.into(),
                    status: CheckStatus::Fail,
                    detail: Some(detail),
                });
            }
        }
    }

    let probe = probe_services(facade.deps()).await;
    if probe.llm_available {
        checks.push(DoctorCheck {
            label: "LLM probe".into(),
            status: CheckStatus::Ok,
            detail: None,
        });
    } else {
        checks.push(DoctorCheck {
            label: "LLM probe".into(),
            status: CheckStatus::Warn,
            detail: Some("indisponible (optionnel headless)".into()),
        });
    }
    if probe.embedding_available {
        checks.push(DoctorCheck {
            label: "embedding probe".into(),
            status: CheckStatus::Ok,
            detail: None,
        });
    } else {
        issues += 1;
        checks.push(DoctorCheck {
            label: "embedding probe".into(),
            status: CheckStatus::Fail,
            detail: Some("indisponible".into()),
        });
    }

    if config.security.block_cloud_llm {
        if let Err(err) = assert_llm_egress_allowed(&config) {
            issues += 1;
            checks.push(DoctorCheck {
                label: "egress LLM".into(),
                status: CheckStatus::Fail,
                detail: Some(err.message),
            });
        } else {
            checks.push(DoctorCheck {
                label: "egress LLM".into(),
                status: CheckStatus::Ok,
                detail: None,
            });
        }
    }

    push_env_token(&mut checks, &config.daemon.token_env, "daemon token");
    push_env_token(&mut checks, &config.gateway.token_env, "gateway token");

    let client = probe_client();
    if config.daemon.enabled {
        let url = format!("http://{}:{}/health", config.daemon.bind, config.daemon.port);
        let detail = probe_service_url("daemon", &url, config.daemon.port).await;
        push_service_check(&mut checks, &mut issues, &detail);
    } else {
        checks.push(DoctorCheck {
            label: "daemon".into(),
            status: CheckStatus::Warn,
            detail: Some("désactivé dans orchestrator.toml".into()),
        });
    }

    if config.gateway.enabled {
        let url = format!("http://{}:{}/health", config.gateway.bind, config.gateway.port);
        let detail = probe_service_url("gateway", &url, config.gateway.port).await;
        push_service_check(&mut checks, &mut issues, &detail);
    } else {
        checks.push(DoctorCheck {
            label: "gateway".into(),
            status: CheckStatus::Warn,
            detail: Some("désactivé dans orchestrator.toml".into()),
        });
    }

    let _ = client;
    let enabled_channels = list_enabled_channels(&config);

    Ok(DoctorReport {
        checks,
        enabled_channels,
        issue_count: issues,
    })
}

async fn bridge_ok(facade: &OrchestratorFacade, command: Command) -> Result<(), String> {
    let response = execute_command(facade, command).await;
    match response {
        Response::Error(err) => Err(format!("[{}] {}", err.kind, err.message)),
        _ => Ok(()),
    }
}

fn push_env_token(checks: &mut Vec<DoctorCheck>, env_name: &str, label: &str) {
    if env_name.is_empty() {
        checks.push(DoctorCheck {
            label: label.into(),
            status: CheckStatus::Warn,
            detail: Some("variable non configurée".into()),
        });
        return;
    }
    match std::env::var(env_name) {
        Ok(_) => checks.push(DoctorCheck {
            label: label.into(),
            status: CheckStatus::Ok,
            detail: Some(env_name.into()),
        }),
        Err(_) => checks.push(DoctorCheck {
            label: label.into(),
            status: CheckStatus::Fail,
            detail: Some(format!("définir {env_name}")),
        }),
    }
}

fn push_service_check(
    checks: &mut Vec<DoctorCheck>,
    issues: &mut usize,
    detail: &crate::harness::types::ServiceStatusDetail,
) {
    let (status, bump) = match detail.state.as_str() {
        "active" => (CheckStatus::Ok, false),
        "down" | "http_error" => (CheckStatus::Warn, false),
        _ => (CheckStatus::Warn, false),
    };
    if bump {
        *issues += 1;
    }
    checks.push(DoctorCheck {
        label: detail.name.clone(),
        status,
        detail: detail.detail.clone(),
    });
}

fn list_enabled_channels(config: &OrchestratorConfig) -> Vec<String> {
    #[cfg(feature = "gateway")]
    {
        use crate::gateway::resolve_channel_config;
        use crate::ChannelCatalog;
        let catalog = ChannelCatalog::new();
        return catalog
            .descriptors()
            .iter()
            .filter(|d| resolve_channel_config(&config.gateway, d.id).enabled)
            .map(|d| d.id.to_string())
            .collect();
    }
    #[cfg(not(feature = "gateway"))]
    {
        let _ = config;
        Vec::new()
    }
}