//! Onboard et reconfiguration workspace.

use std::path::Path;

use crate::config::editor::{set_primary_llm, set_security_profile};
use crate::harness::error::HarnessError;
use crate::harness::types::{ConfigureOptions, OnboardOptions, OnboardResult};
use crate::harness::workspace::{config_path, ensure_workspace_tree, find_example_config, write_minimal_config};
use crate::harness::env::ensure_daemon_token;
use crate::harness::daemon::install_scheduled_task;

fn resolve_profile(options: &OnboardOptions) -> String {
    if options.local_only {
        return "local_only".into();
    }
    if let Some(p) = &options.profile {
        return p.clone();
    }
    "ai_assisted".into()
}

/// Initialise le workspace (structure, config, token, tâche optionnelle).
pub fn run_onboard(workspace: &Path, options: &OnboardOptions) -> Result<OnboardResult, HarnessError> {
    ensure_workspace_tree(workspace)?;

    let settings = config_path(workspace);
    if !settings.exists() {
        if let Some(src) = find_example_config() {
            std::fs::copy(&src, &settings)
                .map_err(|e| HarnessError::io(&settings, e.to_string()))?;
        } else {
            write_minimal_config(&settings)?;
        }
    }

    let profile = resolve_profile(options);
    set_security_profile(&settings, &profile)?;

    let llm = options.llm.clone().or_else(|| {
        if options.local_only || profile == "local_only" {
            Some("ollama".into())
        } else {
            None
        }
    });
    if let Some(ref provider) = llm {
        set_primary_llm(&settings, provider)?;
    }

    let token_new = ensure_daemon_token()?;
    let mut daemon_task_installed = false;
    if options.install_daemon {
        let result = install_scheduled_task(workspace)?;
        daemon_task_installed = result.installed;
    }

    Ok(OnboardResult {
        profile,
        llm,
        daemon_token_generated: token_new,
        daemon_task_installed,
        workspace_display: workspace.display().to_string(),
    })
}

/// Met à jour des champs harness dans orchestrator.toml.
pub fn run_configure(workspace: &Path, options: &ConfigureOptions) -> Result<(), HarnessError> {
    let settings = config_path(workspace);
    if !settings.exists() {
        return Err(HarnessError::MissingConfig { path: settings });
    }

    if options.local_only || options.profile.is_some() {
        let profile = if options.local_only {
            "local_only".to_string()
        } else {
            options
                .profile
                .clone()
                .unwrap_or_else(|| "ai_assisted".into())
        };
        set_security_profile(&settings, &profile)?;
    }

    let llm = options
        .llm
        .as_deref()
        .or(if options.local_only { Some("ollama") } else { None });
    if let Some(provider) = llm {
        set_primary_llm(&settings, provider)?;
    }

    Ok(())
}