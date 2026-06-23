use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;


use crate::config::OrchestratorConfig;
use crate::skills::manifest::{compute_integrity_hash, ManifestError};

/// Entrée du catalogue marketplace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketplaceEntry {
    /// Identifiant stable (répertoire hub cible).
    pub id: String,
    /// Nom affiché.
    pub name: String,
    /// Description.
    pub description: String,
    /// Version catalogue.
    pub version: String,
    /// Installe cette entrée lors d'un `sync`.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Contenu complet du `skill.toml` à écrire.
    pub manifest_toml: String,
}

fn default_true() -> bool {
    true
}

/// Catalogue marketplace (`catalog.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketplaceCatalog {
    /// Version du format catalogue.
    pub version: u32,
    /// Empreinte BLAKE3 du catalogue sans ce champ (Phase 14).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog_hash: Option<String>,
    /// Entrées disponibles.
    pub skills: Vec<MarketplaceEntry>,
}

/// Résultat d'une synchronisation marketplace → hub local.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketplaceSyncResult {
    /// Skills installées ou mises à jour.
    pub installed: Vec<String>,
    /// Skills ignorées (désactivées ou déjà à jour).
    pub skipped: Vec<String>,
}

/// Rapport de vérification d'intégrité des manifestes hub.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrityReport {
    /// Manifestes valides.
    pub valid: Vec<PathBuf>,
    /// Manifestes invalides (hash ou parse).
    pub invalid: Vec<(PathBuf, String)>,
}

/// Erreurs marketplace.
#[derive(Debug, Error)]
pub enum MarketplaceError {
    /// IO disque.
    #[error("io {path}: {message}")]
    Io {
        /// Chemin.
        path: PathBuf,
        /// Détail.
        message: String,
    },
    /// JSON invalide.
    #[error("parse catalogue: {0}")]
    Parse(String),
    /// Signature catalogue invalide ou absente.
    #[error("catalogue non signé ou hash invalide: {0}")]
    UnsignedCatalog(String),
    /// Catalogue distant.
    #[cfg(feature = "skills-marketplace")]
    #[error("fetch distant: {0}")]
    Fetch(String),
    /// Feature absente.
    #[cfg(not(feature = "skills-marketplace"))]
    #[error("sync distant requiert la feature skills-marketplace")]
    FetchDisabled,
    /// Erreur manifeste.
    #[error(transparent)]
    Manifest(#[from] ManifestError),
}

/// Marketplace skills — catalogue local et sync vers le hub.
#[derive(Debug, Clone, Default)]
pub struct SkillsMarketplace;

impl SkillsMarketplace {
    /// Charge le catalogue local configuré.
    ///
    /// # Errors
    ///
    /// Propage [`MarketplaceError`] si le fichier est absent ou invalide.
    pub fn load_catalog(config: &OrchestratorConfig) -> Result<MarketplaceCatalog, MarketplaceError> {
        let path = config.marketplace_catalog_path();
        let raw = std::fs::read_to_string(&path).map_err(|e| MarketplaceError::Io {
            path: path.clone(),
            message: e.to_string(),
        })?;
        let catalog: MarketplaceCatalog =
            serde_json::from_str(&raw).map_err(|e| MarketplaceError::Parse(e.to_string()))?;
        verify_catalog_trust(config, &catalog)?;
        Ok(catalog)
    }

    /// Charge le catalogue distant si `marketplace_url` est défini, sinon local.
    ///
    /// # Errors
    ///
    /// Propage [`MarketplaceError`] en cas d'échec réseau ou parse.
    pub async fn load_catalog_auto(
        config: &OrchestratorConfig,
    ) -> Result<MarketplaceCatalog, MarketplaceError> {
        if !config.skills_hub.marketplace_enabled {
            return Self::load_catalog(config);
        }
        if let Some(url) = &config.skills_hub.marketplace_url {
            if !url.is_empty() {
                #[cfg(feature = "skills-marketplace")]
                {
                    return Self::fetch_catalog(config, url).await;
                }
                #[cfg(not(feature = "skills-marketplace"))]
                {
                    let _ = config;
                    return Self::fetch_catalog(url).await;
                }
            }
        }
        Self::load_catalog(config)
    }

    /// Installe les entrées `enabled` du catalogue dans `workspace/skills/<id>/skill.toml`.
    ///
    /// # Errors
    ///
    /// Propage [`MarketplaceError`] si l'écriture échoue.
    pub fn sync_to_hub(
        config: &OrchestratorConfig,
        catalog: &MarketplaceCatalog,
    ) -> Result<MarketplaceSyncResult, MarketplaceError> {
        let hub = config.skills_hub_dir();
        std::fs::create_dir_all(&hub).map_err(|e| MarketplaceError::Io {
            path: hub.clone(),
            message: e.to_string(),
        })?;

        let mut installed = Vec::new();
        let mut skipped = Vec::new();
        for entry in &catalog.skills {
            if !entry.enabled {
                skipped.push(entry.id.clone());
                continue;
            }
            let skill_dir = hub.join(&entry.id);
            std::fs::create_dir_all(&skill_dir).map_err(|e| MarketplaceError::Io {
                path: skill_dir.clone(),
                message: e.to_string(),
            })?;
            let manifest_path = skill_dir.join("skill.toml");
            let existing = manifest_path.exists().then(|| {
                std::fs::read_to_string(&manifest_path).unwrap_or_default()
            });
            if existing.as_deref() == Some(entry.manifest_toml.as_str()) {
                skipped.push(entry.id.clone());
                continue;
            }
            std::fs::write(&manifest_path, &entry.manifest_toml).map_err(|e| {
                MarketplaceError::Io {
                    path: manifest_path.clone(),
                    message: e.to_string(),
                }
            })?;
            installed.push(entry.id.clone());
        }
        Ok(MarketplaceSyncResult { installed, skipped })
    }

    /// Vérifie les empreintes BLAKE3 des manifestes du hub.
    ///
    /// # Errors
    ///
    /// Propage [`MarketplaceError`] si le scan échoue.
    pub fn verify_hub_integrity(config: &OrchestratorConfig) -> Result<IntegrityReport, MarketplaceError> {
        let hub = config.skills_hub_dir();
        let mut valid = Vec::new();
        let mut invalid = Vec::new();
        if !hub.is_dir() {
            return Ok(IntegrityReport { valid, invalid });
        }
        for entry in std::fs::read_dir(&hub).map_err(|e| MarketplaceError::Io {
            path: hub.clone(),
            message: e.to_string(),
        })? {
            let entry = entry.map_err(|e| MarketplaceError::Io {
                path: hub.clone(),
                message: e.to_string(),
            })?;
            let path = entry.path();
            let manifest = if path.is_dir() {
                path.join("skill.toml")
            } else if path.file_name().and_then(|n| n.to_str()) == Some("skill.toml") {
                path
            } else {
                continue;
            };
            if !manifest.is_file() {
                continue;
            }
            match verify_manifest_file(&manifest) {
                Ok(()) => valid.push(manifest),
                Err(err) => invalid.push((manifest, err.to_string())),
            }
        }
        valid.sort();
        invalid.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(IntegrityReport { valid, invalid })
    }
}

fn verify_manifest_file(path: &Path) -> Result<(), ManifestError> {
    crate::skills::manifest::load_manifest(path).map(|_| ())
}

impl MarketplaceCatalog {
    /// Calcule l'empreinte BLAKE3 recommandée pour un `manifest_toml`.
    #[must_use]
    pub fn hash_manifest(manifest_toml: &str) -> String {
        compute_integrity_hash(manifest_toml)
    }

    /// Calcule l'empreinte BLAKE3 du catalogue (champ `catalog_hash` exclu).
    ///
    /// # Errors
    ///
    /// Propage une erreur de sérialisation JSON.
    pub fn compute_catalog_hash(catalog: &MarketplaceCatalog) -> Result<String, MarketplaceError> {
        let mut unsigned = catalog.clone();
        unsigned.catalog_hash = None;
        let canonical = serde_json::to_string(&unsigned)
            .map_err(|e| MarketplaceError::Parse(e.to_string()))?;
        Ok(blake3::hash(canonical.as_bytes()).to_hex().to_string())
    }

    /// Vérifie le `catalog_hash` présent, si défini.
    ///
    /// # Errors
    ///
    /// Retourne [`MarketplaceError::UnsignedCatalog`] si le hash ne correspond pas.
    pub fn verify_catalog_hash(catalog: &MarketplaceCatalog) -> Result<(), MarketplaceError> {
        let Some(expected) = catalog.catalog_hash.as_ref() else {
            return Ok(());
        };
        let normalized = expected.trim().trim_start_matches("blake3:");
        let actual = Self::compute_catalog_hash(catalog)?;
        if constant_time_eq(actual.as_bytes(), normalized.as_bytes()) {
            Ok(())
        } else {
            Err(MarketplaceError::UnsignedCatalog(format!(
                "attendu {normalized}, obtenu {actual}"
            )))
        }
    }
}

fn verify_catalog_trust(
    config: &OrchestratorConfig,
    catalog: &MarketplaceCatalog,
) -> Result<(), MarketplaceError> {
    if config.skills_hub.marketplace_require_signature && catalog.catalog_hash.is_none() {
        return Err(MarketplaceError::UnsignedCatalog(
            "catalog_hash requis par marketplace_require_signature".into(),
        ));
    }
    MarketplaceCatalog::verify_catalog_hash(catalog)
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right.iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

#[cfg(feature = "skills-marketplace")]
impl SkillsMarketplace {
    async fn fetch_catalog(
        config: &OrchestratorConfig,
        url: &str,
    ) -> Result<MarketplaceCatalog, MarketplaceError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| MarketplaceError::Fetch(e.to_string()))?;
        let raw = client
            .get(url)
            .send()
            .await
            .map_err(|e| MarketplaceError::Fetch(e.to_string()))?
            .error_for_status()
            .map_err(|e| MarketplaceError::Fetch(e.to_string()))?
            .text()
            .await
            .map_err(|e| MarketplaceError::Fetch(e.to_string()))?;
        let catalog: MarketplaceCatalog =
            serde_json::from_str(&raw).map_err(|e| MarketplaceError::Parse(e.to_string()))?;
        verify_catalog_trust(config, &catalog)?;
        Ok(catalog)
    }
}

#[cfg(not(feature = "skills-marketplace"))]
impl SkillsMarketplace {
    async fn fetch_catalog(_url: &str) -> Result<MarketplaceCatalog, MarketplaceError> {
        Err(MarketplaceError::FetchDisabled)
    }
}

/// Suggère des skills par correspondance textuelle (agentic Phase 13).
#[must_use]
pub fn suggest_skills(
    skills: &[crate::skills::SkillEntry],
    query: &str,
    limit: usize,
) -> Vec<crate::skills::SkillEntry> {
    let q = query.to_lowercase();
    let mut scored: Vec<_> = skills
        .iter()
        .filter_map(|entry| {
            let score = score_skill(entry, &q);
            if score > 0 {
                Some((score, entry.clone()))
            } else {
                None
            }
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name.cmp(&b.1.name)));
    scored
        .into_iter()
        .take(limit.max(1))
        .map(|(_, entry)| entry)
        .collect()
}

/// Retourne la skill la mieux notée pour une requête (Phase 14 auto-exécute).
#[must_use]
pub fn best_skill_match(
    skills: &[crate::skills::SkillEntry],
    query: &str,
) -> Option<(u32, crate::skills::SkillEntry)> {
    let q = query.to_lowercase();
    skills
        .iter()
        .filter_map(|entry| {
            let score = score_skill(entry, &q);
            if score > 0 {
                Some((score, entry.clone()))
            } else {
                None
            }
        })
        .max_by_key(|(score, entry)| (*score, std::cmp::Reverse(entry.name.clone())))
}

fn score_skill(entry: &crate::skills::SkillEntry, query: &str) -> u32 {
    let name = entry.name.to_lowercase();
    let description = entry.description.to_lowercase();
    let mut score = 0u32;
    if name.contains(query) {
        score += 10;
    }
    if description.contains(query) {
        score += 5;
    }
    for token in query.split_whitespace().filter(|t| t.len() >= 3) {
        if name.contains(token) {
            score += 3;
        }
        if description.contains(token) {
            score += 1;
        }
    }
    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::{SkillEntry, SkillSource};

    #[test]
    fn suggest_skills_ranks_by_relevance() {
        let skills = vec![
            SkillEntry {
                name: "pong".into(),
                description: "Plugin démo pong".into(),
                source: SkillSource::Hub,
                version: None,
            },
            SkillEntry {
                name: "search".into(),
                description: "Recherche mémoire".into(),
                source: SkillSource::Builtin,
                version: None,
            },
        ];
        let hits = suggest_skills(&skills, "pong plugin", 5);
        assert_eq!(hits.first().map(|s| s.name.as_str()), Some("pong"));
    }

    #[test]
    fn sync_writes_marketplace_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let catalog_path = dir.path().join("catalog.json");
        let manifest = r#"
[skill]
id = "market-echo"
description = "Echo marketplace"

[subprocess]
command = "echo"
args = ["market"]
"#;
        let hash = MarketplaceCatalog::hash_manifest(manifest);
        let manifest_with_hash = format!(
            "[skill]\nid = \"market-echo\"\ndescription = \"Echo marketplace\"\nintegrity_hash = \"{hash}\"\n\n[subprocess]\ncommand = \"echo\"\nargs = [\"market\"]\n"
        );
        let catalog = MarketplaceCatalog {
            version: 1,
            catalog_hash: None,
            skills: vec![MarketplaceEntry {
                id: "market-echo".into(),
                name: "Market Echo".into(),
                description: "Echo".into(),
                version: "0.1.0".into(),
                enabled: true,
                manifest_toml: manifest_with_hash,
            }],
        };
        std::fs::write(&catalog_path, serde_json::to_string(&catalog).unwrap()).unwrap();

        let mut config = OrchestratorConfig::default();
        config.workspace_root = dir.path().to_path_buf();
        config.skills_hub.marketplace_catalog = "catalog.json".into();

        let loaded = SkillsMarketplace::load_catalog(&config).unwrap();
        let result = SkillsMarketplace::sync_to_hub(&config, &loaded).unwrap();
        assert_eq!(result.installed, vec!["market-echo"]);
        assert!(dir.path().join("skills/market-echo/skill.toml").is_file());
    }

    #[test]
    fn catalog_hash_roundtrip() {
        let catalog = MarketplaceCatalog {
            version: 1,
            catalog_hash: None,
            skills: vec![MarketplaceEntry {
                id: "demo".into(),
                name: "Demo".into(),
                description: "Test".into(),
                version: "0.1.0".into(),
                enabled: true,
                manifest_toml: "[skill]\nid=\"demo\"".into(),
            }],
        };
        let hash = MarketplaceCatalog::compute_catalog_hash(&catalog).unwrap();
        let signed = MarketplaceCatalog {
            catalog_hash: Some(hash.clone()),
            ..catalog
        };
        MarketplaceCatalog::verify_catalog_hash(&signed).expect("hash valide");
        let mut config = OrchestratorConfig::default();
        config.skills_hub.marketplace_require_signature = true;
        assert!(verify_catalog_trust(&config, &signed).is_ok());
        let unsigned = MarketplaceCatalog {
            catalog_hash: None,
            ..signed
        };
        assert!(verify_catalog_trust(&config, &unsigned).is_err());
    }
}