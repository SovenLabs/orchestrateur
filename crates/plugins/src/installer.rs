use std::path::{Path, PathBuf};

use orchestrator::{
    MarketplaceCatalog, MarketplaceSyncResult, OrchestratorConfig, SkillsMarketplace,
};
use thiserror::Error;
use tracing::info;

/// Résultat d'installation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallResult {
    /// Identifiant installé.
    pub skill_id: String,
    /// Chemin destination.
    pub target_dir: PathBuf,
    /// Sync marketplace si applicable.
    pub sync: Option<MarketplaceSyncResult>,
}

/// Erreurs d'installation.
#[derive(Debug, Error)]
pub enum InstallError {
    /// IO disque.
    #[error("io: {0}")]
    Io(String),
    /// Marketplace.
    #[error("marketplace: {0}")]
    Marketplace(String),
    /// Skill absente du catalogue.
    #[error("skill `{0}` absente du catalogue")]
    NotInCatalog(String),
    /// Source invalide.
    #[error("source invalide: {0}")]
    InvalidSource(String),
}

/// Installe des skills depuis un dossier local ou le marketplace (Phase 6).
#[derive(Debug, Clone, Default)]
pub struct SkillInstaller;

impl SkillInstaller {
    /// Copie un répertoire skill (`skill.toml` à la racine) vers le hub.
    ///
    /// # Errors
    ///
    /// Propage [`InstallError`] si la copie échoue.
    pub fn install_from_directory(
        config: &OrchestratorConfig,
        source_dir: &Path,
        skill_id: Option<&str>,
    ) -> Result<InstallResult, InstallError> {
        let manifest = source_dir.join("skill.toml");
        if !manifest.is_file() {
            return Err(InstallError::InvalidSource(format!(
                "skill.toml absent dans {}",
                source_dir.display()
            )));
        }
        let id = skill_id.map(str::to_string).unwrap_or_else(|| {
            source_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("skill")
                .to_string()
        });
        let target = config.skills_hub_dir().join(&id);
        copy_dir_recursive(source_dir, &target).map_err(|e| InstallError::Io(e.to_string()))?;
        info!(%id, path = %target.display(), "skill installée depuis dossier");
        Ok(InstallResult {
            skill_id: id,
            target_dir: target,
            sync: None,
        })
    }

    /// Installe depuis le catalogue marketplace (local ou distant).
    ///
    /// # Errors
    ///
    /// Propage [`InstallError`] si le catalogue ou la sync échoue.
    pub async fn install_from_marketplace(
        config: &OrchestratorConfig,
        skill_id: &str,
    ) -> Result<InstallResult, InstallError> {
        let catalog = SkillsMarketplace::load_catalog_auto(config)
            .await
            .map_err(|e| InstallError::Marketplace(e.to_string()))?;
        if !catalog.skills.iter().any(|s| s.id == skill_id) {
            return Err(InstallError::NotInCatalog(skill_id.to_string()));
        }
        let filtered = MarketplaceCatalog {
            version: catalog.version,
            catalog_hash: catalog.catalog_hash,
            skills: catalog
                .skills
                .into_iter()
                .filter(|s| s.id == skill_id)
                .collect(),
        };
        let sync = SkillsMarketplace::sync_to_hub(config, &filtered)
            .map_err(|e| InstallError::Marketplace(e.to_string()))?;
        Ok(InstallResult {
            skill_id: skill_id.to_string(),
            target_dir: config.skills_hub_dir().join(skill_id),
            sync: Some(sync),
        })
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let target = dst.join(file_name);
        if path.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            std::fs::copy(&path, &target)?;
        }
    }
    Ok(())
}