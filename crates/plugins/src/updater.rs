use orchestrator::{
    MarketplaceCatalog, MarketplaceSyncResult, OrchestratorConfig, SkillsMarketplace,
};
use thiserror::Error;
use tracing::info;

/// Rapport de mise à jour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateReport {
    /// Skills mises à jour ou installées.
    pub updated: Vec<String>,
    /// Skills ignorées.
    pub skipped: Vec<String>,
}

/// Erreurs de mise à jour.
#[derive(Debug, Error)]
pub enum UpdaterError {
    /// Marketplace.
    #[error("marketplace: {0}")]
    Marketplace(String),
}

/// Met à jour les skills depuis le catalogue marketplace (Phase 6).
#[derive(Debug, Clone, Default)]
pub struct SkillUpdater;

impl SkillUpdater {
    /// Synchronise tout le catalogue vers le hub local.
    ///
    /// # Errors
    ///
    /// Propage [`UpdaterError`] si le chargement ou la sync échoue.
    pub async fn update_all(config: &OrchestratorConfig) -> Result<UpdateReport, UpdaterError> {
        let catalog = SkillsMarketplace::load_catalog_auto(config)
            .await
            .map_err(|e| UpdaterError::Marketplace(e.to_string()))?;
        let sync = SkillsMarketplace::sync_to_hub(config, &catalog)
            .map_err(|e| UpdaterError::Marketplace(e.to_string()))?;
        info!(
            installed = sync.installed.len(),
            skipped = sync.skipped.len(),
            "skills marketplace synchronisées"
        );
        Ok(sync_into_report(sync))
    }

    /// Met à jour une skill précise.
    ///
    /// # Errors
    ///
    /// Propage [`UpdaterError`] si la sync échoue.
    pub async fn update_one(
        config: &OrchestratorConfig,
        skill_id: &str,
    ) -> Result<UpdateReport, UpdaterError> {
        let catalog = SkillsMarketplace::load_catalog_auto(config)
            .await
            .map_err(|e| UpdaterError::Marketplace(e.to_string()))?;
        let filtered = MarketplaceCatalog {
            version: catalog.version,
            catalog_hash: catalog.catalog_hash,
            skills: catalog
                .skills
                .into_iter()
                .filter(|s| s.id == skill_id)
                .collect(),
        };
        if filtered.skills.is_empty() {
            return Err(UpdaterError::Marketplace(format!(
                "skill `{skill_id}` absente du catalogue"
            )));
        }
        let sync = SkillsMarketplace::sync_to_hub(config, &filtered)
            .map_err(|e| UpdaterError::Marketplace(e.to_string()))?;
        Ok(sync_into_report(sync))
    }
}

fn sync_into_report(sync: MarketplaceSyncResult) -> UpdateReport {
    UpdateReport {
        updated: sync.installed,
        skipped: sync.skipped,
    }
}