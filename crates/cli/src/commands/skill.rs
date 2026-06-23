//! `orch skill` — skills opérationnelles et hub.

use anyhow::Result;
use clap::Subcommand;
use orchestrator::{
    BridgeSkillContext, Command, MarketplaceCatalog, OrchestratorFacade, SkillsHub,
    SkillsMarketplace,
};

use crate::context::run_bridge_command;

/// Sous-commandes skills.
#[derive(Debug, Clone, Subcommand)]
pub enum SkillCommands {
    /// Liste les skills enregistrées.
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
    /// Chemin du répertoire hub.
    #[command(hide = true)]
    Path,
    /// Catalogue marketplace.
    #[command(hide = true)]
    Marketplace,
    /// Vérifie les empreintes BLAKE3 du hub.
    #[command(hide = true)]
    Verify,
}

pub async fn run(cmd: SkillCommands, facade: &OrchestratorFacade) -> Result<()> {
    match cmd {
        SkillCommands::List => run_bridge_command(facade, Command::ListSkills).await,
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
        SkillCommands::Update => sync_marketplace(facade).await,
        SkillCommands::Path => {
            println!("{}", facade.deps().config.skills_hub_dir().display());
            Ok(())
        }
        SkillCommands::Marketplace => print_marketplace(facade).await,
        SkillCommands::Verify => verify_hub(facade),
    }
}

async fn install_skill(facade: &OrchestratorFacade, name: &str) -> Result<()> {
    let config = &facade.deps().config;
    let catalog = SkillsMarketplace::load_catalog_auto(config)
        .await
        .map_err(anyhow::Error::msg)?;
    if !catalog.skills.iter().any(|s| s.id == name) {
        anyhow::bail!("skill `{name}` absente du catalogue marketplace");
    }
    let filtered: Vec<_> = catalog
        .skills
        .iter()
        .filter(|s| s.id == name)
        .cloned()
        .collect();
    let mini = MarketplaceCatalog {
        version: catalog.version,
        catalog_hash: catalog.catalog_hash,
        skills: filtered,
    };
    let result = SkillsMarketplace::sync_to_hub(config, &mini).map_err(anyhow::Error::msg)?;
    println!("Installé : {:?}", result.installed);
    Ok(())
}

async fn sync_marketplace(facade: &OrchestratorFacade) -> Result<()> {
    let config = &facade.deps().config;
    let catalog = SkillsMarketplace::load_catalog_auto(config)
        .await
        .map_err(anyhow::Error::msg)?;
    let result = SkillsMarketplace::sync_to_hub(config, &catalog).map_err(anyhow::Error::msg)?;
    println!(
        "Sync : {} installée(s), {} ignorée(s)",
        result.installed.len(),
        result.skipped.len()
    );
    Ok(())
}

async fn print_marketplace(facade: &OrchestratorFacade) -> Result<()> {
    let config = &facade.deps().config;
    let catalog = SkillsMarketplace::load_catalog_auto(config)
        .await
        .map_err(anyhow::Error::msg)?;
    println!("# Marketplace v{} ({} skills)", catalog.version, catalog.skills.len());
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