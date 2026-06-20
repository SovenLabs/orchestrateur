use std::path::{Path, PathBuf};

use cortex::SimilarityThresholds;
use serde::Deserialize;
use thiserror::Error;

/// Configuration applicative de l'orchestrateur.
#[derive(Debug, Clone, PartialEq)]
pub struct OrchestratorConfig {
    /// Racine du workspace utilisateur (mémoires, config, vecteurs).
    pub workspace_root: PathBuf,
    /// Seuils pour le calcul des backlinks sémantiques.
    pub similarity_thresholds: SimilarityThresholds,
    /// Dimension des embeddings (utilisée par les mocks ; ignorée par les adapters réels).
    pub embedding_dim: usize,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            workspace_root: PathBuf::from("workspace"),
            similarity_thresholds: SimilarityThresholds::default(),
            embedding_dim: 8,
        }
    }
}

impl OrchestratorConfig {
    /// Chemin du répertoire des mémoires Markdown.
    #[must_use]
    pub fn memories_dir(&self) -> PathBuf {
        self.workspace_root.join("memories")
    }

    /// Chemin du fichier de configuration TOML.
    #[must_use]
    pub fn settings_path(&self) -> PathBuf {
        self.workspace_root.join("config").join("orchestrator.toml")
    }

    /// Charge la configuration depuis `workspace/config/orchestrator.toml`.
    ///
    /// Retombe sur les valeurs par défaut si le fichier est absent.
    ///
    /// # Errors
    ///
    /// Retourne [`ConfigError`] si le fichier existe mais est illisible ou invalide.
    pub fn load_workspace(workspace_root: impl Into<PathBuf>) -> Result<Self, ConfigError> {
        let mut config = Self {
            workspace_root: workspace_root.into(),
            similarity_thresholds: SimilarityThresholds::default(),
            embedding_dim: 8,
        };
        let path = config.settings_path();
        if path.exists() {
            config.apply_toml_file(&path)?;
        }
        Ok(config)
    }

    /// Applique les valeurs d'un fichier TOML sur cette configuration.
    ///
    /// # Errors
    ///
    /// Retourne [`ConfigError`] en cas d'erreur de lecture ou de parsing.
    pub fn apply_toml_file(&mut self, path: &Path) -> Result<(), ConfigError> {
        let raw = std::fs::read_to_string(path).map_err(|e| ConfigError::Io {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;
        let settings: SettingsToml = toml::from_str(&raw).map_err(|e| ConfigError::Parse {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;
        self.merge_settings(settings);
        Ok(())
    }

    fn merge_settings(&mut self, settings: SettingsToml) {
        if let Some(ws) = settings.workspace {
            if let Some(root) = ws.root {
                self.workspace_root = PathBuf::from(root);
            }
        }
        if let Some(bl) = settings.backlinks {
            if let Some(v) = bl.semantic_min {
                self.similarity_thresholds.semantic_min = v;
            }
            if let Some(v) = bl.max_links {
                self.similarity_thresholds.max_links = v;
            }
        }
    }
}

/// Erreurs de chargement de configuration.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    /// Erreur d'accès disque.
    #[error("config IO {path:?}: {message}")]
    Io {
        /// Chemin concerné.
        path: PathBuf,
        /// Détail de l'erreur.
        message: String,
    },
    /// TOML invalide.
    #[error("config parse {path:?}: {message}")]
    Parse {
        /// Chemin concerné.
        path: PathBuf,
        /// Détail de l'erreur.
        message: String,
    },
}

#[derive(Debug, Deserialize)]
struct SettingsToml {
    workspace: Option<WorkspaceSection>,
    backlinks: Option<BacklinksSection>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceSection {
    root: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BacklinksSection {
    semantic_min: Option<f32>,
    max_links: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn default_config_has_sane_thresholds() {
        let cfg = OrchestratorConfig::default();
        assert!((cfg.similarity_thresholds.semantic_min - 0.75).abs() < f32::EPSILON);
        assert_eq!(cfg.similarity_thresholds.max_links, 10);
        assert_eq!(cfg.embedding_dim, 8);
    }

    #[test]
    fn loads_backlinks_from_toml() {
        let dir = tempfile::tempdir().unwrap();
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&config_dir).unwrap();
        let toml_path = config_dir.join("orchestrator.toml");
        let mut file = std::fs::File::create(&toml_path).unwrap();
        let root = dir.path().to_string_lossy().replace('\\', "/");
        writeln!(
            file,
            r#"
[workspace]
root = "{root}"

[backlinks]
semantic_min = 0.8
max_links = 5
"#
        )
        .unwrap();

        let cfg = OrchestratorConfig::load_workspace(dir.path()).unwrap();
        assert!((cfg.similarity_thresholds.semantic_min - 0.8).abs() < f32::EPSILON);
        assert_eq!(cfg.similarity_thresholds.max_links, 5);
    }

    #[test]
    fn missing_toml_uses_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = OrchestratorConfig::load_workspace(dir.path()).unwrap();
        assert_eq!(cfg.similarity_thresholds.max_links, 10);
    }
}