use std::path::PathBuf;

use orchestrator::{OrchestratorConfig, SkillHubDescriptor, SkillLoader, SkillMetadata, SkillType};
use thiserror::Error;

/// Erreurs du registre local installé.
#[derive(Debug, Error)]
pub enum RegistryError {
    /// Erreur hub / loader.
    #[error("{0}")]
    Hub(String),
}

/// Skill installée dans le hub local.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledSkill {
    /// Descripteur hub.
    pub descriptor: SkillHubDescriptor,
    /// Métadonnées enrichies.
    pub metadata: SkillMetadata,
}

/// Registre des skills installées localement (Phase 6).
#[derive(Debug, Clone, Default)]
pub struct InstalledSkillRegistry;

impl InstalledSkillRegistry {
    /// Liste toutes les skills du hub workspace.
    ///
    /// # Errors
    ///
    /// Propage [`RegistryError`] si la découverte échoue.
    pub fn list(config: &OrchestratorConfig) -> Result<Vec<InstalledSkill>, RegistryError> {
        let descriptors = SkillLoader::discover(config)
            .map_err(|e| RegistryError::Hub(e.to_string()))?;
        let metadata_map = SkillLoader::collect_metadata(&descriptors);
        Ok(descriptors
            .into_iter()
            .map(|descriptor| {
                let metadata = metadata_map
                    .get(&descriptor.id)
                    .cloned()
                    .unwrap_or_else(|| {
                        SkillMetadata::minimal(&descriptor.id, &descriptor.description)
                    });
                InstalledSkill {
                    descriptor,
                    metadata,
                }
            })
            .collect())
    }

    /// Filtre par type fonctionnel.
    ///
    /// # Errors
    ///
    /// Propage [`RegistryError`] si la découverte échoue.
    pub fn list_by_type(
        config: &OrchestratorConfig,
        skill_type: SkillType,
    ) -> Result<Vec<InstalledSkill>, RegistryError> {
        Ok(Self::list(config)?
            .into_iter()
            .filter(|s| s.metadata.skill_type == skill_type)
            .collect())
    }

    /// Chemin racine du hub.
    #[must_use]
    pub fn hub_dir(config: &OrchestratorConfig) -> PathBuf {
        config.skills_hub_dir()
    }
}