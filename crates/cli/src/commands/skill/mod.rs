//! `orch skill` — hub, marketplace, exécution.

mod create;
mod install;
mod list;

use anyhow::Result;
use clap::Subcommand;
use orchestrateur_plugins::SkillUpdater;
use orchestrator::{BridgeSkillContext, Command, OrchestratorFacade, SkillsHub, SkillsMarketplace};

use crate::context::run_bridge_command;

pub use install::{from_directory as install_from_dir, from_marketplace as install_skill};
pub use list::run as list_skills;

/// Sous-commandes skills.
#[derive(Debug, Clone, Subcommand)]
pub enum SkillCommands {
    /// Liste les skills installées.
    List,
    /// Exécute une skill par nom.
    Run {
        name: String,
        #[arg(long)]
        query: Option<String>,
        #[arg(long)]
        text: Option<String>,
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Installe une skill depuis le marketplace (sync ciblé).
    Install {
        name: String,
    },
    /// Synchronise toutes les skills du catalogue marketplace.
    Update,
    /// Génère un nouveau skill (template skill.toml + SKILL.md).
    Create {
        id: String,
        #[arg(long, default_value = "Nouvelle skill Orchestrateur")]
        description: String,
        #[arg(long, default_value = "subprocess")]
        kind: String,
        #[arg(long, default_value = "generic")]
        skill_type: String,
        #[arg(long)]
        author: Option<String>,
    },
    /// Installe depuis un dossier local contenant skill.toml.
    InstallDir {
        path: std::path::PathBuf,
        #[arg(long)]
        id: Option<String>,
    },
    #[command(hide = true)]
    Path,
    #[command(hide = true)]
    Marketplace,
    #[command(hide = true)]
    Verify,
}

pub async fn run(cmd: SkillCommands, facade: &OrchestratorFacade) -> Result<()> {
    match cmd {
        SkillCommands::List => list_skills(facade).await,
        SkillCommands::Run {
            name,
            query,
            text,
            tags,
            limit,
        } => {
            run_bridge_command(
                facade,
                Command::ExecuteSkill {
                    name,
                    context: BridgeSkillContext {
                        query,
                        text,
                        tags,
                        limit,
                    },
                },
            )
            .await
        }
        SkillCommands::Install { name } => install_skill(facade, &name).await,
        SkillCommands::InstallDir { path, id } => {
            install_from_dir(facade, &path, id.as_deref()).await
        }
        SkillCommands::Update => sync_marketplace(facade).await,
        SkillCommands::Create {
            id,
            description,
            kind,
            skill_type,
            author,
        } => create::run(
            facade,
            &id,
            &description,
            &kind,
            &skill_type,
            author.as_deref(),
        ),
        SkillCommands::Path => {
            println!("{}", facade.deps().config.skills_hub_dir().display());
            Ok(())
        }
        SkillCommands::Marketplace => print_marketplace(facade).await,
        SkillCommands::Verify => verify_hub(facade),
    }
}

async fn sync_marketplace(facade: &OrchestratorFacade) -> Result<()> {
    let report = SkillUpdater::update_all(&facade.deps().config)
        .await
        .map_err(anyhow::Error::msg)?;
    println!(
        "Sync : {} mise(s) à jour, {} ignorée(s)",
        report.updated.len(),
        report.skipped.len()
    );
    Ok(())
}

async fn print_marketplace(facade: &OrchestratorFacade) -> Result<()> {
    let config = &facade.deps().config;
    let catalog = SkillsMarketplace::load_catalog_auto(config)
        .await
        .map_err(anyhow::Error::msg)?;
    println!(
        "# Marketplace v{} ({} skills)",
        catalog.version,
        catalog.skills.len()
    );
    for entry in &catalog.skills {
        println!("{} — {}", entry.id, entry.description);
    }
    Ok(())
}

fn verify_hub(facade: &OrchestratorFacade) -> Result<()> {
    let report = SkillsMarketplace::verify_hub_integrity(&facade.deps().config)
        .map_err(anyhow::Error::msg)?;
    println!(
        "Intégrité : {} ok, {} erreur(s)",
        report.valid.len(),
        report.invalid.len()
    );
    if !report.invalid.is_empty() {
        anyhow::bail!("manifestes invalides");
    }
    Ok(())
}

/// Liste hub filesystem (legacy skills-hub list).
pub fn list_hub(facade: &OrchestratorFacade) -> Result<()> {
    let config = &facade.deps().config;
    let entries = SkillsHub::discover(config).map_err(anyhow::Error::msg)?;
    if entries.is_empty() {
        println!("Aucune skill dans le hub.");
        return Ok(());
    }
    println!("# Skills hub ({})", entries.len());
    for entry in entries {
        println!("{} [{}] — {}", entry.id, entry.kind, entry.description);
    }
    Ok(())
}