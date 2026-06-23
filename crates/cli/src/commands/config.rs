//! `orch config` — configuration orchestrator.toml.

use std::path::Path;

use anyhow::Result;
use clap::Subcommand;
use orchestrator::{
    set_primary_llm, set_security_profile, OrchestratorConfig, ProviderRegistry,
};

use crate::tui;

fn settings_path(workspace: &Path) -> Result<std::path::PathBuf> {
    let path = orchestrator::harness::config_path(workspace);
    if !path.exists() {
        anyhow::bail!(
            "config absente — lancez `orch onboard` ({})",
            path.display()
        );
    }
    Ok(path)
}

#[derive(Debug, Clone, Subcommand)]
pub enum ConfigCommands {
    Get,
    Set {
        key: String,
        value: String,
    },
    Edit,
}

pub fn run(cmd: ConfigCommands, workspace: &Path) -> Result<()> {
    match cmd {
        ConfigCommands::Get => print_config(workspace),
        ConfigCommands::Set { key, value } => set_config(workspace, &key, &value),
        ConfigCommands::Edit => tui::run_settings(workspace),
    }
}

fn print_config(workspace: &Path) -> Result<()> {
    let config = OrchestratorConfig::load_workspace(workspace)
        .map_err(|e| anyhow::anyhow!("config: {e}"))?;
    println!("# orchestrator.toml");
    println!("workspace     : {}", workspace.display());
    println!("llm           : {}", config.providers.primary_llm);
    println!("embedding     : {}", config.providers.primary_embedding);
    if let Some(profile) = &config.security.profile {
        println!("profile       : {profile:?}");
    }
    println!("block_cloud   : {}", config.security.block_cloud_llm);
    println!(
        "daemon        : {}:{} (enabled={})",
        config.daemon.bind, config.daemon.port, config.daemon.enabled
    );
    println!(
        "gateway       : {}:{} (enabled={})",
        config.gateway.bind, config.gateway.port, config.gateway.enabled
    );
    Ok(())
}

fn set_config(workspace: &Path, key: &str, value: &str) -> Result<()> {
    let settings = settings_path(workspace)?;
    match key {
        "llm" => {
            validate_llm(value)?;
            set_primary_llm(&settings, value).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("LLM primaire : {value}");
        }
        "profile" => {
            set_security_profile(&settings, value).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Profil sécurité : {value}");
        }
        other => anyhow::bail!("clé inconnue: {other} (utiliser llm ou profile)"),
    }
    Ok(())
}

fn validate_llm(provider: &str) -> Result<()> {
    let registry = ProviderRegistry::new();
    if registry.llm_descriptors().iter().any(|d| d.id == provider) {
        Ok(())
    } else {
        anyhow::bail!("provider LLM inconnu: {provider}")
    }
}

