//! Sauvegarde et restauration du workspace utilisateur.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use walkdir::WalkDir;

use crate::config::OrchestratorConfig;
use crate::VERSION;

/// Erreurs backup / restore.
#[derive(Debug, Error)]
pub enum BackupError {
    /// Erreur disque.
    #[error("backup IO {path:?}: {message}")]
    Io {
        /// Chemin concerné.
        path: PathBuf,
        /// Détail.
        message: String,
    },
    /// Manifeste absent ou illisible.
    #[error("manifeste backup invalide: {0}")]
    Manifest(String),
    /// Archive incompatible.
    #[error("backup incompatible: {0}")]
    Incompatible(String),
}

/// Manifeste d'une sauvegarde Orchestrateur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupManifest {
    /// Horodatage UTC ISO-8601.
    pub created_at: String,
    /// Version binaire au moment de la sauvegarde.
    pub orchestrator_version: String,
    /// Chemins relatifs copiés depuis la racine workspace.
    pub paths: Vec<String>,
}

/// Résultat d'une sauvegarde.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupReport {
    /// Dossier de sauvegarde créé.
    pub backup_dir: PathBuf,
    /// Nombre de fichiers copiés.
    pub files_copied: usize,
    /// Manifeste écrit.
    pub manifest: BackupManifest,
}

/// Chemins relatifs sauvegardés par défaut.
#[must_use]
pub fn default_backup_paths(config: &OrchestratorConfig) -> Vec<PathBuf> {
    vec![
        PathBuf::from("memories"),
        PathBuf::from("config"),
        PathBuf::from("agents"),
        PathBuf::from("registry"),
        PathBuf::from("skills"),
        PathBuf::from("b212"),
        PathBuf::from("logs"),
        PathBuf::from(".orchestrateur"),
        config.security.audit.path.clone(),
    ]
}

/// Crée une sauvegarde complète du workspace dans `dest_parent/orchestrateur-backup-<timestamp>/`.
///
/// # Errors
///
/// Retourne [`BackupError`] en cas d'échec I/O.
pub fn create_backup(
    config: &OrchestratorConfig,
    dest_parent: &Path,
) -> Result<BackupReport, BackupError> {
    let stamp = Utc::now().format("%Y%m%dT%H%M%SZ");
    let backup_dir = dest_parent.join(format!("orchestrateur-backup-{stamp}"));
    fs::create_dir_all(&backup_dir).map_err(|e| BackupError::Io {
        path: backup_dir.clone(),
        message: e.to_string(),
    })?;

    let workspace = &config.workspace_root;
    let mut files_copied = 0usize;
    let mut paths = Vec::new();

    for rel in default_backup_paths(config) {
        let src = workspace.join(&rel);
        if !src.exists() {
            continue;
        }
        let dst = backup_dir.join(&rel);
        copy_tree(&src, &dst, &mut files_copied)?;
        paths.push(rel.to_string_lossy().into_owned());
    }

    let manifest = BackupManifest {
        created_at: Utc::now().to_rfc3339(),
        orchestrator_version: VERSION.to_string(),
        paths,
    };
    let manifest_path = backup_dir.join("backup-manifest.json");
    let json = serde_json::to_string_pretty(&manifest).map_err(|e| BackupError::Manifest(e.to_string()))?;
    fs::write(&manifest_path, json).map_err(|e| BackupError::Io {
        path: manifest_path,
        message: e.to_string(),
    })?;
    files_copied += 1;

    Ok(BackupReport {
        backup_dir,
        files_copied,
        manifest,
    })
}

/// Restaure un workspace depuis un dossier de sauvegarde.
///
/// Les fichiers existants sont écrasés. Crée la racine workspace si absente.
///
/// # Errors
///
/// Retourne [`BackupError`] si le manifeste est absent ou si la copie échoue.
pub fn restore_backup(backup_dir: &Path, workspace: &Path) -> Result<BackupManifest, BackupError> {
    let manifest_path = backup_dir.join("backup-manifest.json");
    let raw = fs::read_to_string(&manifest_path).map_err(|e| BackupError::Manifest(e.to_string()))?;
    let manifest: BackupManifest =
        serde_json::from_str(&raw).map_err(|e| BackupError::Manifest(e.to_string()))?;

    fs::create_dir_all(workspace).map_err(|e| BackupError::Io {
        path: workspace.to_path_buf(),
        message: e.to_string(),
    })?;

    for rel in &manifest.paths {
        let src = backup_dir.join(rel);
        if !src.exists() {
            continue;
        }
        let dst = workspace.join(rel);
        let mut count = 0usize;
        copy_tree(&src, &dst, &mut count)?;
    }

    Ok(manifest)
}

fn copy_tree(src: &Path, dst: &Path, counter: &mut usize) -> Result<(), BackupError> {
    if src.is_file() {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(|e| BackupError::Io {
                path: parent.to_path_buf(),
                message: e.to_string(),
            })?;
        }
        fs::copy(src, dst).map_err(|e| BackupError::Io {
            path: src.to_path_buf(),
            message: e.to_string(),
        })?;
        *counter += 1;
        return Ok(());
    }

    if !src.is_dir() {
        return Ok(());
    }

    fs::create_dir_all(dst).map_err(|e| BackupError::Io {
        path: dst.to_path_buf(),
        message: e.to_string(),
    })?;

    for entry in WalkDir::new(src).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        let rel = path.strip_prefix(src).map_err(|e| BackupError::Io {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).map_err(|e| BackupError::Io {
                path: target.clone(),
                message: e.to_string(),
            })?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| BackupError::Io {
                    path: parent.to_path_buf(),
                    message: e.to_string(),
                })?;
            }
            fs::copy(path, &target).map_err(|e| BackupError::Io {
                path: path.to_path_buf(),
                message: e.to_string(),
            })?;
            *counter += 1;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OrchestratorConfig;

    #[test]
    fn roundtrip_backup_restore_preserves_memories() {
        let workspace = tempfile::tempdir().unwrap();
        let backups = tempfile::tempdir().unwrap();
        let mut config = OrchestratorConfig::default();
        config.workspace_root = workspace.path().to_path_buf();

        let memories = config.memories_dir();
        fs::create_dir_all(&memories).unwrap();
        let mem_file = memories.join("test-memory.md");
        fs::write(&mem_file, "# test\ncontenu").unwrap();

        let report = create_backup(&config, backups.path()).unwrap();
        assert!(report.files_copied >= 1);

        let restore_root = tempfile::tempdir().unwrap();
        let manifest = restore_backup(&report.backup_dir, restore_root.path()).unwrap();
        assert!(!manifest.paths.is_empty());
        let restored = restore_root.path().join("memories/test-memory.md");
        assert!(restored.exists());
        let content = fs::read_to_string(restored).unwrap();
        assert!(content.contains("contenu"));
    }
}