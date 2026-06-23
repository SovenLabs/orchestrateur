//! `orch skill list` — skills installées dans le hub.

use anyhow::Result;
use orchestrateur_plugins::InstalledSkillRegistry;
use orchestrator::OrchestratorFacade;

pub async fn run(facade: &OrchestratorFacade) -> Result<()> {
    let config = &facade.deps().config;
    let installed = InstalledSkillRegistry::list(config).map_err(anyhow::Error::msg)?;
    if installed.is_empty() {
        println!("Aucune skill installée.");
        return Ok(());
    }
    println!("# Skills ({})", installed.len());
    for skill in installed {
        println!(
            "{} [{}] v{} — {}",
            skill.descriptor.id,
            format!("{:?}", skill.metadata.skill_type).to_ascii_lowercase(),
            skill.descriptor.version,
            skill.descriptor.description
        );
        if !skill.metadata.dependencies.is_empty() {
            println!("  deps: {}", skill.metadata.dependencies.join(", "));
        }
    }
    Ok(())
}