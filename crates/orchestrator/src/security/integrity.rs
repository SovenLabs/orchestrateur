//! Couche 2 — intégrité cryptographique de la configuration.

use std::fs;
use std::path::{Path, PathBuf};

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::OrchestratorConfig;

/// Statut d'intégrité au démarrage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrityStatus {
    /// Configuration vérifiée ou vérification désactivée.
    Healthy,
    /// Modification détectée — mode dégradé.
    Degraded {
        /// Raison lisible pour l'humain.
        reason: String,
    },
}

impl IntegrityStatus {
    /// Indique si le système est en mode dégradé.
    #[must_use]
    pub fn is_degraded(&self) -> bool {
        matches!(self, Self::Degraded { .. })
    }

    /// Raison du mode dégradé, si applicable.
    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Healthy => None,
            Self::Degraded { reason } => Some(reason.as_str()),
        }
    }
}

/// Manifeste d'intégrité BLAKE3 pour `orchestrator.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityManifest {
    /// Algorithme de hachage.
    pub algorithm: String,
    /// Empreinte hexadécimale.
    pub hash: String,
    /// Fichier source hashé.
    pub source: String,
}

/// Erreur de vérification d'intégrité.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum IntegrityError {
    /// Accès disque impossible.
    #[error("intégrité IO {path:?}: {message}")]
    Io {
        /// Chemin concerné.
        path: PathBuf,
        /// Détail.
        message: String,
    },
    /// Manifeste illisible.
    #[error("manifeste d'intégrité invalide: {0}")]
    InvalidManifest(String),
}

/// Vérifie l'intégrité du fichier de configuration.
///
/// Si le manifeste est absent et `bootstrap_on_missing` est actif, il est créé (trust-on-first-use).
///
/// # Errors
///
/// Retourne [`IntegrityError`] uniquement en cas d'erreur IO critique.
pub fn verify_config_integrity(
    config: &OrchestratorConfig,
) -> Result<IntegrityStatus, IntegrityError> {
    let integrity = &config.security.integrity;
    if !integrity.enabled || !integrity.verify_config_hash {
        return Ok(IntegrityStatus::Healthy);
    }

    let settings_path = config.settings_path();
    if !settings_path.exists() {
        return Ok(IntegrityStatus::Healthy);
    }

    let content = fs::read_to_string(&settings_path).map_err(|e| IntegrityError::Io {
        path: settings_path.clone(),
        message: e.to_string(),
    })?;
    let hash = hash_bytes(content.as_bytes());
    let manifest_path = integrity_manifest_path(&settings_path);

    if !manifest_path.exists() {
        if integrity.bootstrap_on_missing {
            write_manifest(&manifest_path, &hash)?;
            tracing::info!(
                path = %manifest_path.display(),
                "manifeste d'intégrité créé (trust-on-first-use)"
            );
            return Ok(IntegrityStatus::Healthy);
        }
        if integrity.require_manifest {
            return Ok(IntegrityStatus::Degraded {
                reason: "manifeste d'intégrité absent".into(),
            });
        }
        return Ok(IntegrityStatus::Healthy);
    }

    let manifest = read_manifest(&manifest_path)?;
    if manifest.hash != hash {
        tracing::error!(
            expected = %manifest.hash,
            actual = %hash,
            "altération détectée sur orchestrator.toml"
        );
        return Ok(IntegrityStatus::Degraded {
            reason: "empreinte BLAKE3 de orchestrator.toml invalide".into(),
        });
    }

    Ok(IntegrityStatus::Healthy)
}

/// Chemin du manifeste d'intégrité adjacent au TOML.
#[must_use]
pub fn integrity_manifest_path(settings_path: &Path) -> PathBuf {
    let file_name = settings_path.file_name().map_or_else(
        || "orchestrator.toml.integrity.json".into(),
        |f| format!("{}.integrity.json", f.to_string_lossy()),
    );
    settings_path
        .parent()
        .map_or_else(|| PathBuf::from(&file_name), |p| p.join(&file_name))
}

fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

fn write_manifest(path: &Path, hash: &str) -> Result<(), IntegrityError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| IntegrityError::Io {
            path: parent.to_path_buf(),
            message: e.to_string(),
        })?;
    }
    let manifest = IntegrityManifest {
        algorithm: "blake3".into(),
        hash: hash.to_string(),
        source: "orchestrator.toml".into(),
    };
    let json = serde_json::to_string_pretty(&manifest).map_err(|e| {
        IntegrityError::InvalidManifest(e.to_string())
    })?;
    fs::write(path, json).map_err(|e| IntegrityError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}

fn read_manifest(path: &Path) -> Result<IntegrityManifest, IntegrityError> {
    let raw = fs::read_to_string(path).map_err(|e| IntegrityError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    serde_json::from_str(&raw).map_err(|e| IntegrityError::InvalidManifest(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_config_tampering() {
        let dir = tempfile::tempdir().expect("tempdir");
        let config_dir = dir.path().join("config");
        fs::create_dir_all(&config_dir).expect("mkdir");
        let toml_path = config_dir.join("orchestrator.toml");
        fs::write(&toml_path, "[workspace]\npath = \"./m\"\n").expect("write");

        let mut cfg = OrchestratorConfig::load_workspace(dir.path()).expect("load");
        cfg.workspace_root = dir.path().to_path_buf();
        cfg.security.integrity.enabled = true;
        cfg.security.integrity.verify_config_hash = true;
        cfg.security.integrity.bootstrap_on_missing = true;

        let status = verify_config_integrity(&cfg).expect("bootstrap");
        assert_eq!(status, IntegrityStatus::Healthy);

        fs::write(&toml_path, "[workspace]\npath = \"./tampered\"\n").expect("tamper");
        let status = verify_config_integrity(&cfg).expect("verify");
        assert!(status.is_degraded());
    }
}