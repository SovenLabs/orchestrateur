//! `orch skill install` — marketplace et dossier local.

use std::path::Path;

use anyhow::Result;
use orchestrateur_plugins::SkillInstaller;
use orchestrator::OrchestratorFacade;

pub async fn from_marketplace(facade: &OrchestratorFacade, name: &str) -> Result<()> {
    let config = &facade.deps().config;
    let result = SkillInstaller::install_from_marketplace(config, name)
        .await
        .map_err(anyhow::Error::msg)?;
    println!(
        "Installé : {} → {}",
        result.skill_id,
        result.target_dir.display()
    );
    Ok(())
}

pub async fn from_directory(
    facade: &OrchestratorFacade,
    path: &Path,
    id: Option<&str>,
) -> Result<()> {
    let result = SkillInstaller::install_from_directory(&facade.deps().config, path, id)
        .map_err(anyhow::Error::msg)?;
    println!(
        "Installé : {} → {}",
        result.skill_id,
        result.target_dir.display()
    );
    Ok(())
}