use std::path::{Path, PathBuf};

use thiserror::Error;
use tracing::warn;

use crate::config::{OrchestratorConfig, SkillsHubEntryConfig};
use crate::skills::manifest::{load_manifest, ManifestError, SkillManifest, SkillPluginConfig};
use crate::skills::plugin::SubprocessPluginSkill;
use crate::skills::registry::SkillRegistry;

#[cfg(feature = "plugins-native")]
use crate::skills::native::NativePluginSkill;

/// Descripteur hub (découverte sans chargement).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillHubDescriptor {
    /// Identifiant stable.
    pub id: String,
    /// Nom affiché.
    pub name: String,
    /// Description.
    pub description: String,
    /// Version.
    pub version: String,
    /// Type plugin (`subprocess` / `native`).
    pub kind: String,
    /// Origine (`filesystem` ou `inline`).
    pub origin: String,
    /// Chemin du manifeste ou entrée inline.
    pub path: PathBuf,
    /// Skill activée.
    pub enabled: bool,
}

/// Erreurs du hub de skills.
#[derive(Debug, Error)]
pub enum HubError {
    /// Erreur manifeste.
    #[error(transparent)]
    Manifest(#[from] ManifestError),
    /// Erreur IO scan.
    #[error("scan hub: {0}")]
    Scan(String),
}

/// Hub filesystem + entrées inline pour plugins dynamiques.
#[derive(Debug, Clone, Default)]
pub struct SkillsHub;

impl SkillsHub {
    /// Découvre les skills sans les charger dans le registre.
    ///
    /// # Errors
    ///
    /// Propage [`HubError`] si le scan du répertoire échoue.
    pub fn discover(config: &OrchestratorConfig) -> Result<Vec<SkillHubDescriptor>, HubError> {
        let mut entries = Vec::new();
        for (idx, entry) in config.skills_hub.entries.iter().enumerate() {
            entries.push(descriptor_from_inline(entry, idx));
        }
        let hub_dir = config.skills_hub_dir();
        if hub_dir.is_dir() {
            entries.extend(scan_directory(&hub_dir)?);
        }
        entries.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(entries)
    }

    /// Charge les plugins hub actifs dans le registre.
    ///
    /// # Errors
    ///
    /// Propage [`HubError`] si le scan échoue. Les manifestes invalides sont ignorés avec avertissement.
    pub fn load_into(registry: &mut SkillRegistry, config: &OrchestratorConfig) -> Result<usize, HubError> {
        let mut loaded = 0usize;
        for entry in &config.skills_hub.entries {
            if !entry.enabled {
                continue;
            }
            registry.register(std::sync::Arc::new(SubprocessPluginSkill::from_entry(
                entry.clone(),
            )));
            loaded += 1;
        }

        let hub_dir = config.skills_hub_dir();
        if !hub_dir.is_dir() {
            return Ok(loaded);
        }

        for manifest_path in find_manifests(&hub_dir)? {
            match load_manifest(&manifest_path) {
                Ok(manifest) => {
                    if !manifest.enabled {
                        continue;
                    }
                    if register_manifest(registry, manifest) {
                        loaded += 1;
                    }
                }
                Err(err) => {
                    warn!(%err, path = %manifest_path.display(), "manifeste hub ignoré");
                }
            }
        }
        Ok(loaded)
    }
}

/// Enregistre un manifeste dans le registre (subprocess ou natif).
pub fn register_manifest(registry: &mut SkillRegistry, manifest: SkillManifest) -> bool {
    match manifest.plugin {
        SkillPluginConfig::Subprocess(_) => {
            registry.register(std::sync::Arc::new(SubprocessPluginSkill::from_manifest(
                manifest,
            )));
            true
        }
        SkillPluginConfig::Native(_) => {
            #[cfg(feature = "plugins-native")]
            {
                match NativePluginSkill::from_manifest(manifest) {
                    Ok(skill) => {
                        registry.register(std::sync::Arc::new(skill));
                        true
                    }
                    Err(err) => {
                        warn!(%err, "plugin natif ignoré");
                        false
                    }
                }
            }
            #[cfg(not(feature = "plugins-native"))]
            {
                warn!(
                    id = %manifest.id,
                    "plugin natif ignoré — recompiler avec feature plugins-native"
                );
                false
            }
        }
    }
}

fn descriptor_from_inline(entry: &SkillsHubEntryConfig, idx: usize) -> SkillHubDescriptor {
    SkillHubDescriptor {
        id: entry.id.clone(),
        name: entry.id.clone(),
        description: entry.description.clone(),
        version: "inline".into(),
        kind: "subprocess".into(),
        origin: "inline".into(),
        path: PathBuf::from(format!("orchestrator.toml#skills_hub.entries[{idx}]")),
        enabled: entry.enabled,
    }
}

fn scan_directory(hub_dir: &Path) -> Result<Vec<SkillHubDescriptor>, HubError> {
    let mut descriptors = Vec::new();
    for manifest_path in find_manifests(hub_dir)? {
        match load_manifest(&manifest_path) {
            Ok(manifest) => descriptors.push(descriptor_from_manifest(&manifest, &manifest_path)),
            Err(err) => {
                warn!(%err, path = %manifest_path.display(), "manifeste hub ignoré");
            }
        }
    }
    Ok(descriptors)
}

fn descriptor_from_manifest(manifest: &SkillManifest, path: &Path) -> SkillHubDescriptor {
    SkillHubDescriptor {
        id: manifest.id.clone(),
        name: manifest.name.clone(),
        description: manifest.description.clone(),
        version: manifest.version.clone(),
        kind: match manifest.plugin {
            SkillPluginConfig::Subprocess(_) => "subprocess".into(),
            SkillPluginConfig::Native(_) => "native".into(),
        },
        origin: "filesystem".into(),
        path: path.to_path_buf(),
        enabled: manifest.enabled,
    }
}

fn find_manifests(hub_dir: &Path) -> Result<Vec<PathBuf>, HubError> {
    let mut paths = Vec::new();
    let read_dir = std::fs::read_dir(hub_dir).map_err(|e| HubError::Scan(e.to_string()))?;
    for entry in read_dir {
        let entry = entry.map_err(|e| HubError::Scan(e.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            let manifest = path.join("skill.toml");
            if manifest.is_file() {
                paths.push(manifest);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some("skill.toml") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}