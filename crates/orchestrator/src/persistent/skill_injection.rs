use std::path::{Path, PathBuf};

use tracing::debug;

use crate::config::OrchestratorConfig;
use crate::error::{OrchestratorError, SkillError};
use crate::persistent::{PersistentAgent, PersistentAgentError};
use crate::{
    load_manifest, register_manifest, SkillHubDescriptor, SkillLoader, SkillPluginConfig,
    SkillRegistry, SkillType,
};

/// Injection dynamique de skills par agent persistant (Phase 6).
#[derive(Debug, Clone, Default)]
pub struct AgentSkillInjector;

impl AgentSkillInjector {
    /// Répertoire skills d'un agent (`workspace/agents/{id}/skills/`).
    #[must_use]
    pub fn agent_skills_dir(agent_root: &Path) -> PathBuf {
        agent_root.join("skills")
    }

    /// Liste les skills déclarées pour un agent (symlinks ou manifestes locaux).
    ///
    /// # Errors
    ///
    /// Propage [`PersistentAgentError`] si la lecture disque échoue.
    pub fn list_for_agent(
        agent: &PersistentAgent,
        config: &OrchestratorConfig,
    ) -> Result<Vec<SkillHubDescriptor>, PersistentAgentError> {
        let mut descriptors = Vec::new();
        let agent_skills = Self::agent_skills_dir(&agent.root);
        if agent_skills.is_dir() {
            descriptors.extend(scan_agent_skills(&agent_skills)?);
        }

        let global = SkillLoader::discover(config).map_err(map_loader_err)?;
        for entry in global {
            if !entry.enabled {
                continue;
            }
            if let Ok(manifest) = load_manifest(&entry.path) {
                if manifest.skill_type == SkillType::Agent
                    && (manifest.agent_ids.is_empty()
                        || manifest.agent_ids.iter().any(|id| id == &agent.config.id))
                {
                    descriptors.push(entry);
                }
            }
        }
        descriptors.sort_by(|a, b| a.id.cmp(&b.id));
        descriptors.dedup_by(|a, b| a.id == b.id);
        Ok(descriptors)
    }

    /// Charge les skills applicables à un agent dans le registre global.
    ///
    /// # Errors
    ///
    /// Propage [`PersistentAgentError`] si le scan échoue.
    pub fn load_for_agent(
        registry: &mut SkillRegistry,
        agent: &PersistentAgent,
        config: &OrchestratorConfig,
    ) -> Result<usize, PersistentAgentError> {
        let descriptors = Self::list_for_agent(agent, config)?;
        let mut loaded = 0usize;
        for descriptor in descriptors {
            if register_manifest(
                registry,
                load_manifest(&descriptor.path).map_err(map_manifest_err)?,
            ) {
                loaded += 1;
                debug!(agent = %agent.config.id, skill = %descriptor.id, "skill agent injectée");
            }
        }
        Ok(loaded)
    }
}

fn map_loader_err(err: crate::LoaderError) -> PersistentAgentError {
    PersistentAgentError::Orchestrator(OrchestratorError::Skill(SkillError::ExecutionFailed(
        err.to_string(),
    )))
}

fn map_manifest_err(err: crate::ManifestError) -> PersistentAgentError {
    PersistentAgentError::Orchestrator(OrchestratorError::Skill(SkillError::ExecutionFailed(
        err.to_string(),
    )))
}

fn scan_agent_skills(dir: &Path) -> Result<Vec<SkillHubDescriptor>, PersistentAgentError> {
    let mut out = Vec::new();
    let read_dir = std::fs::read_dir(dir).map_err(|e| PersistentAgentError::Io(e.to_string()))?;
    for entry in read_dir {
        let entry = entry.map_err(|e| PersistentAgentError::Io(e.to_string()))?;
        let path = entry.path();
        let manifest_path = if path.is_dir() {
            path.join("skill.toml")
        } else if path.file_name().and_then(|n| n.to_str()) == Some("skill.toml") {
            path
        } else {
            continue;
        };
        if !manifest_path.is_file() {
            continue;
        }
        let manifest = load_manifest(&manifest_path).map_err(map_manifest_err)?;
        out.push(SkillHubDescriptor {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            description: manifest.description.clone(),
            version: manifest.version.clone(),
            kind: match manifest.plugin {
                SkillPluginConfig::Subprocess(_) => "subprocess".into(),
                SkillPluginConfig::Native(_) => "native".into(),
            },
            origin: "agent".into(),
            path: manifest_path,
            enabled: manifest.enabled,
        });
    }
    Ok(out)
}