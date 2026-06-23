use std::collections::HashMap;
use std::path::Path;

use thiserror::Error;
use tracing::warn;

use crate::config::OrchestratorConfig;
use crate::skills::dependencies::{resolve_load_order, DependencyError};
use crate::skills::hub::{HubError, SkillHubDescriptor, SkillsHub};
use crate::skills::manifest::{load_manifest, SkillManifest};
use crate::skills::metadata::SkillMetadata;
use crate::skills::registry::SkillRegistry;

/// Erreurs du chargeur de skills.
#[derive(Debug, Error)]
pub enum LoaderError {
    /// Erreur hub.
    #[error(transparent)]
    Hub(#[from] HubError),
    /// Erreur dépendances.
    #[error(transparent)]
    Dependency(#[from] DependencyError),
}

/// Chargeur dynamique des skills hub avec résolution de dépendances (Phase 6).
#[derive(Debug, Clone, Default)]
pub struct SkillLoader;

impl SkillLoader {
    /// Découvre les skills sans chargement.
    ///
    /// # Errors
    ///
    /// Propage [`LoaderError`] si le scan échoue.
    pub fn discover(config: &OrchestratorConfig) -> Result<Vec<SkillHubDescriptor>, LoaderError> {
        Ok(SkillsHub::discover(config)?)
    }

    /// Charge les manifestes et construit la carte de métadonnées.
    #[must_use]
    pub fn collect_metadata(
        descriptors: &[SkillHubDescriptor],
    ) -> HashMap<String, SkillMetadata> {
        let mut map = HashMap::new();
        for descriptor in descriptors {
            if !descriptor.enabled {
                continue;
            }
            if let Ok(manifest) = load_manifest(&descriptor.path) {
                map.insert(descriptor.id.clone(), metadata_from_manifest(&manifest));
            }
        }
        map
    }

    /// Charge les skills hub actives dans l'ordre topologique des dépendances.
    ///
    /// # Errors
    ///
    /// Propage [`LoaderError`] si le scan ou la résolution échoue.
    pub fn load_into(
        registry: &mut SkillRegistry,
        config: &OrchestratorConfig,
    ) -> Result<usize, LoaderError> {
        let descriptors = Self::discover(config)?;
        let metadata = Self::collect_metadata(&descriptors);
        let order = resolve_load_order(&descriptors, &metadata)?;

        let mut loaded = 0usize;
        for id in order {
            let Some(descriptor) = descriptors.iter().find(|d| d.id == id) else {
                continue;
            };
            if register_from_path(registry, &descriptor.path) {
                loaded += 1;
            }
        }

        if config.skills_hub.enabled {
            loaded += load_inline_entries(registry, config)?;
        }

        Ok(loaded)
    }
}

fn metadata_from_manifest(manifest: &SkillManifest) -> SkillMetadata {
    SkillMetadata::from_manifest(
        &manifest.id,
        &manifest.name,
        &manifest.description,
        &manifest.version,
        manifest.author.clone(),
        manifest.skill_type,
        manifest.dependencies.clone(),
        manifest.agent_ids.clone(),
        match manifest.kind {
            crate::skills::manifest::SkillPluginKind::Subprocess => {
                crate::skills::skill::SkillSource::Hub
            }
            crate::skills::manifest::SkillPluginKind::Native => {
                crate::skills::skill::SkillSource::Native
            }
        },
    )
}

fn register_from_path(registry: &mut SkillRegistry, manifest_path: &Path) -> bool {
    match load_manifest(manifest_path) {
        Ok(manifest) if manifest.enabled => crate::skills::hub::register_manifest(registry, manifest),
        Ok(_) => false,
        Err(err) => {
            warn!(%err, path = %manifest_path.display(), "manifeste hub ignoré");
            false
        }
    }
}

fn load_inline_entries(
    registry: &mut SkillRegistry,
    config: &OrchestratorConfig,
) -> Result<usize, LoaderError> {
    let mut loaded = 0usize;
    for entry in &config.skills_hub.entries {
        if !entry.enabled {
            continue;
        }
        registry.register(std::sync::Arc::new(
            crate::skills::plugin::SubprocessPluginSkill::from_entry(entry.clone()),
        ));
        loaded += 1;
    }
    Ok(loaded)
}