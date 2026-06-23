//! `orch skill create` — scaffold skill.toml + SKILL.md.

use anyhow::Result;
use orchestrateur_plugins::{SkillManifestTemplate, SkillTemplateKind};
use orchestrator::{OrchestratorFacade, SkillType};

pub fn run(
    facade: &OrchestratorFacade,
    id: &str,
    description: &str,
    kind: &str,
    skill_type: &str,
    author: Option<&str>,
) -> Result<()> {
    if id.is_empty() || id.contains(['/', '\\', ' ']) {
        anyhow::bail!("identifiant invalide — utilisez un slug sans espaces");
    }
    let template = SkillManifestTemplate {
        id: id.to_string(),
        description: description.to_string(),
        skill_type: SkillType::parse(skill_type),
        plugin_kind: SkillTemplateKind::parse(kind),
        author: author.map(str::to_string),
        dependencies: Vec::new(),
    };
    let hub = facade.deps().config.skills_hub_dir();
    template
        .write_to(&hub)
        .map_err(|e| anyhow::anyhow!("écriture skill: {e}"))?;
    println!(
        "Skill créée : {}/{} (skill.toml + SKILL.md)",
        hub.display(),
        id
    );
    Ok(())
}