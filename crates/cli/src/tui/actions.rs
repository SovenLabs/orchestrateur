//! Exécution des actions menu (bridge harness + prompts).

use std::path::Path;

use anyhow::Result;
use dialoguer::Input;
use infrastructure::bootstrap_workspace;
use orchestrator::{
    execute_command, ChannelCatalog, Command as BridgeCommand, OrchestratorFacade,
    ProviderKind, ProviderRegistry, SkillsHub, SkillsMarketplace,
};
use tokio::runtime::Handle;

use crate::harness_ops::{
    channels_status, cmd_configure, cmd_doctor,
    cmd_harness_run, cmd_harness_smoke, cmd_uninstall, daemon_install, daemon_status,
    HarnessSmokeOptions,
    daemon_stop, gateway_status, providers_set, providers_test, ConfigureOptions,
};
use crate::output::print_response;
use crate::commands::update::{run as run_update, UpdateArgs};

use super::menus::HarnessAction;

async fn bootstrap_facade(workspace: &Path) -> Result<OrchestratorFacade> {
    let deps = bootstrap_workspace(workspace)
        .await
        .map_err(|err| anyhow::anyhow!(err.with_context("menu")))?;
    Ok(OrchestratorFacade::new(deps))
}

async fn run_bridge(facade: &OrchestratorFacade, command: BridgeCommand) -> Result<()> {
    let response = execute_command(facade, command).await;
    print_response(response)
}

/// Exécute une action harness depuis le menu.
pub async fn run_harness_action(workspace: &Path, action: HarnessAction) -> Result<()> {
    match action {
        HarnessAction::Doctor => {
            let facade = bootstrap_facade(workspace).await?;
            cmd_doctor(&facade, workspace).await?;
        }
        HarnessAction::DaemonStatus => daemon_status(workspace).await?,
        HarnessAction::DaemonInstall | HarnessAction::DaemonInstallSettings => {
            daemon_install(workspace)?;
        }
        HarnessAction::DaemonStop => daemon_stop()?,
        HarnessAction::GatewayStatus => gateway_status(workspace).await?,
        HarnessAction::HarnessSmoke => {
            let facade = bootstrap_facade(workspace).await?;
            cmd_harness_smoke(&facade, workspace, &HarnessSmokeOptions::default()).await?;
        }
        HarnessAction::HarnessRun => cmd_harness_run(workspace).await?,
        HarnessAction::Health => {
            let facade = bootstrap_facade(workspace).await?;
            run_bridge(&facade, BridgeCommand::HealthCheck).await?;
        }
        HarnessAction::Search => {
            let query: String = Input::new()
                .with_prompt("Requête de recherche")
                .interact_text()?;
            let facade = bootstrap_facade(workspace).await?;
            run_bridge(
                &facade,
                BridgeCommand::Search {
                    query,
                    limit: 10,
                },
            )
            .await?;
        }
        HarnessAction::ListMemories => {
            let facade = bootstrap_facade(workspace).await?;
            run_bridge(
                &facade,
                BridgeCommand::List {
                    filter: None,
                    offset: 0,
                    limit: 50,
                },
            )
            .await?;
        }
        HarnessAction::Assimilate => {
            let text: String = Input::new()
                .with_prompt("Texte à assimiler")
                .interact_text()?;
            let facade = bootstrap_facade(workspace).await?;
            run_bridge(
                &facade,
                BridgeCommand::Assimilate {
                    text,
                    tags: vec![],
                },
            )
            .await?;
        }
        HarnessAction::Chat => {
            let message: String = Input::new()
                .with_prompt("Message chat")
                .interact_text()?;
            let facade = bootstrap_facade(workspace).await?;
            run_bridge(&facade, BridgeCommand::Chat { message }).await?;
        }
        HarnessAction::Graph => {
            let facade = bootstrap_facade(workspace).await?;
            run_bridge(&facade, BridgeCommand::Graph).await?;
        }
        HarnessAction::DraftList => {
            let facade = bootstrap_facade(workspace).await?;
            run_bridge(&facade, BridgeCommand::ListDrafts).await?;
        }
        HarnessAction::ImportMd => {
            let source: String = Input::new()
                .with_prompt("Dossier source (.md)")
                .interact_text()?;
            let facade = bootstrap_facade(workspace).await?;
            let result = facade.import_from_directory(Path::new(&source)).await?;
            println!(
                "Import : {} ok, {} ignorées, {} erreurs",
                result.imported,
                result.skipped,
                result.errors.len()
            );
        }
        HarnessAction::Reindex => {
            let facade = bootstrap_facade(workspace).await?;
            let memories = facade.list_memories().await?;
            let mut ok = 0usize;
            for memory in &memories {
                if facade.save_memory(memory).await.is_ok() {
                    ok += 1;
                }
            }
            println!("Ré-indexation : {ok}/{} mémoire(s).", memories.len());
        }
        HarnessAction::ChannelsList => {
            let catalog = ChannelCatalog::new();
            println!("# Canaux ({})", catalog.count());
            for ch in catalog.descriptors() {
                println!("  {} — {}", ch.id, ch.display_name);
            }
        }
        HarnessAction::ChannelsStatus => channels_status(workspace)?,
        HarnessAction::SkillsList => {
            let facade = bootstrap_facade(workspace).await?;
            let response = execute_command(&facade, BridgeCommand::ListSkills).await;
            print_response(response)?;
        }
        HarnessAction::SkillRun => {
            let name: String = Input::new()
                .with_prompt("Identifiant skill")
                .interact_text()?;
            let facade = bootstrap_facade(workspace).await?;
            let response = execute_command(
                &facade,
                BridgeCommand::ExecuteSkill {
                    name,
                    context: Default::default(),
                },
            )
            .await;
            print_response(response)?;
        }
        HarnessAction::SkillsHubList
        | HarnessAction::SkillsHubPath
        | HarnessAction::SkillsHubMarketplace
        | HarnessAction::SkillsHubSync
        | HarnessAction::SkillsHubVerify => {
            let facade = bootstrap_facade(workspace).await?;
            skills_hub_action(&facade, action).await?;
        }
        HarnessAction::ProvidersList => print_providers_list(None)?,
        HarnessAction::ProvidersTest => {
            let facade = bootstrap_facade(workspace).await?;
            providers_test(&facade, None).await?;
        }
        HarnessAction::ProviderSetOllama => providers_set(workspace, "ollama")?,
        HarnessAction::ProviderSetXai => providers_set(workspace, "xai")?,
        HarnessAction::ConfigureLocalOnly => {
            cmd_configure(
                workspace,
                &ConfigureOptions {
                    profile: Some("local_only".into()),
                    llm: Some("ollama".into()),
                    local_only: true,
                },
            )?;
        }
        HarnessAction::ShowConfigPath => {
            let path = workspace.join("config").join("orchestrator.toml");
            println!("Workspace : {}", workspace.display());
            println!("Config    : {}", path.display());
            if path.exists() {
                println!("(fichier présent)");
            } else {
                println!("(fichier absent — lancez onboard)");
            }
        }
        HarnessAction::Update => {
            run_update(UpdateArgs::default()).await?;
        }
        HarnessAction::Uninstall => cmd_uninstall()?,
        HarnessAction::Audit => {
            let facade = bootstrap_facade(workspace).await?;
            run_bridge(
                &facade,
                BridgeCommand::Audit { limit: 50 },
            )
            .await?;
        }
        HarnessAction::WatcherStatus => {
            let facade = bootstrap_facade(workspace).await?;
            run_bridge(&facade, BridgeCommand::WatcherStatus).await?;
        }
        HarnessAction::WatcherStart => {
            let facade = bootstrap_facade(workspace).await?;
            run_bridge(&facade, BridgeCommand::WatcherStart).await?;
        }
        HarnessAction::WatcherStop => {
            let facade = bootstrap_facade(workspace).await?;
            run_bridge(&facade, BridgeCommand::WatcherStop).await?;
        }
        HarnessAction::McpInfo => {
            println!("Serveur MCP stdio :");
            println!("  orchestrateur mcp serve --workspace \"{}\"", workspace.display());
            println!("Ajoutez cette commande dans Cursor / Claude Code (MCP).");
        }
    }
    Ok(())
}

async fn skills_hub_action(facade: &OrchestratorFacade, action: HarnessAction) -> Result<()> {
    let config = &facade.deps().config;
    match action {
        HarnessAction::SkillsHubPath => {
            println!("{}", config.skills_hub_dir().display());
        }
        HarnessAction::SkillsHubList => {
            let entries = SkillsHub::discover(config).map_err(anyhow::Error::msg)?;
            for entry in entries {
                println!("- {} : {}", entry.id, entry.description);
            }
        }
        HarnessAction::SkillsHubMarketplace => {
            let catalog = SkillsMarketplace::load_catalog_auto(config)
                .await
                .map_err(anyhow::Error::msg)?;
            for entry in &catalog.skills {
                println!("- {} : {}", entry.id, entry.description);
            }
        }
        HarnessAction::SkillsHubSync => {
            let catalog = SkillsMarketplace::load_catalog_auto(config)
                .await
                .map_err(anyhow::Error::msg)?;
            let result = SkillsMarketplace::sync_to_hub(config, &catalog).map_err(anyhow::Error::msg)?;
            println!(
                "Sync : {} installée(s), {} ignorée(s)",
                result.installed.len(),
                result.skipped.len()
            );
        }
        HarnessAction::SkillsHubVerify => {
            let report = SkillsMarketplace::verify_hub_integrity(config).map_err(anyhow::Error::msg)?;
            println!(
                "Intégrité : {} ok, {} invalide(s)",
                report.valid.len(),
                report.invalid.len()
            );
        }
        _ => {}
    }
    Ok(())
}

fn print_providers_list(kind: Option<&str>) -> Result<()> {
    let registry = ProviderRegistry::new();
    match kind {
        Some("llm") => print_provider_table(registry.llm_descriptors()),
        Some("embedding") => print_provider_table(registry.embedding_descriptors()),
        Some(other) => anyhow::bail!("kind inconnu: {other}"),
        None => {
            println!("# LLM");
            print_provider_table(registry.llm_descriptors());
            println!("# Embeddings");
            print_provider_table(registry.embedding_descriptors());
        }
    }
    Ok(())
}

fn print_provider_table(descriptors: &[orchestrator::ProviderDescriptor]) {
    for d in descriptors {
        let kind = match d.kind {
            ProviderKind::Llm => "llm",
            ProviderKind::Embedding => "embedding",
        };
        println!("  {:<14} {:<12} {}", d.id, kind, d.display_name);
    }
}

/// Bloque sur une action async depuis la boucle menu synchrone.
pub fn block_on_action(workspace: &Path, action: HarnessAction) -> Result<()> {
    Handle::current().block_on(run_harness_action(workspace, action))
}