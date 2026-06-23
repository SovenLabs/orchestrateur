//! Statut canaux gateway.

use std::path::Path;

use crate::config::OrchestratorConfig;
use crate::config::editor::set_channel_enabled;
use crate::gateway::resolve_channel_config;
use crate::harness::error::HarnessError;
use crate::harness::types::ChannelStatusRow;
use crate::harness::workspace::config_path;
use crate::ChannelCatalog;

/// Active un canal dans orchestrator.toml.
pub fn enable_channel(workspace: &Path, channel_id: &str) -> Result<(), HarnessError> {
    let settings = config_path(workspace);
    if !settings.exists() {
        return Err(HarnessError::MissingConfig { path: settings });
    }
    set_channel_enabled(&settings, channel_id, true)?;
    Ok(())
}

/// Désactive un canal.
pub fn disable_channel(workspace: &Path, channel_id: &str) -> Result<(), HarnessError> {
    let settings = config_path(workspace);
    if !settings.exists() {
        return Err(HarnessError::MissingConfig { path: settings });
    }
    set_channel_enabled(&settings, channel_id, false)?;
    Ok(())
}

/// Liste le statut de tous les canaux.
pub fn list_channel_status(workspace: &Path) -> Result<Vec<ChannelStatusRow>, HarnessError> {
    let config = OrchestratorConfig::load_workspace(workspace)?;
    let catalog = ChannelCatalog::new();
    let rows = catalog
        .descriptors()
        .iter()
        .map(|descriptor| {
            let cfg = resolve_channel_config(&config.gateway, descriptor.id);
            let token_state = if cfg.token_env.is_empty() {
                "n/a".to_string()
            } else if std::env::var(&cfg.token_env).is_ok() {
                format!("{}=set", cfg.token_env)
            } else {
                format!("{}=missing", cfg.token_env)
            };
            ChannelStatusRow {
                id: descriptor.id.to_string(),
                enabled: cfg.enabled,
                display_name: descriptor.display_name.to_string(),
                token_env: cfg.token_env.clone(),
                token_state,
                kind: if descriptor.dedicated {
                    "live".into()
                } else {
                    "stub".into()
                },
            }
        })
        .collect();
    Ok(rows)
}