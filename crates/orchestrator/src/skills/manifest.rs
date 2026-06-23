use std::path::{Path, PathBuf};

use blake3::Hasher;
use serde::Deserialize;
use thiserror::Error;

/// Type de plugin skill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillPluginKind {
    /// Exécute une commande subprocess.
    Subprocess,
    /// Charge une bibliothèque dynamique (Phase 12).
    Native,
}

/// Configuration subprocess d'un plugin.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SubprocessPluginConfig {
    /// Commande exécutable.
    pub command: String,
    /// Arguments.
    pub args: Vec<String>,
    /// Envoie le contexte skill en JSON sur stdin.
    pub stdin_json: bool,
    /// Timeout en secondes.
    pub timeout_secs: u64,
}

/// Configuration plugin natif.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativePluginConfig {
    /// Chemin bibliothèque (relatif au répertoire skill ou absolu).
    pub library: String,
}

/// Configuration effective selon le `kind`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillPluginConfig {
    /// Plugin subprocess.
    Subprocess(SubprocessPluginConfig),
    /// Plugin natif `.dll` / `.so`.
    Native(NativePluginConfig),
}

/// Manifeste TOML d'une skill hub (`skill.toml`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillManifest {
    /// Identifiant stable.
    pub id: String,
    /// Nom affiché.
    pub name: String,
    /// Description lisible.
    pub description: String,
    /// Version sémantique libre.
    pub version: String,
    /// Type de plugin.
    pub kind: SkillPluginKind,
    /// Active le chargement.
    pub enabled: bool,
    /// Configuration plugin.
    pub plugin: SkillPluginConfig,
    /// Répertoire source du manifeste.
    pub root: PathBuf,
}

/// Erreurs de parsing / découverte des manifestes.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ManifestError {
    /// Lecture disque.
    #[error("io {path}: {message}")]
    Io {
        /// Chemin concerné.
        path: PathBuf,
        /// Détail.
        message: String,
    },
    /// TOML invalide.
    #[error("parse {path}: {message}")]
    Parse {
        /// Chemin concerné.
        path: PathBuf,
        /// Détail.
        message: String,
    },
    /// Manifeste incomplet.
    #[error("manifeste invalide {path}: {message}")]
    Invalid {
        /// Chemin concerné.
        path: PathBuf,
        /// Détail.
        message: String,
    },
}

/// Charge un manifeste depuis `skill.toml`.
///
/// # Errors
///
/// Propage [`ManifestError`] si le fichier est absent, illisible ou invalide.
pub fn load_manifest(path: &Path) -> Result<SkillManifest, ManifestError> {
    let raw = std::fs::read_to_string(path).map_err(|e| ManifestError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    let parsed: SkillManifestToml = toml::from_str(&raw).map_err(|e| ManifestError::Parse {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    if let Some(hash) = parsed
        .skill
        .as_ref()
        .and_then(|s| s.integrity_hash.as_deref())
    {
        verify_integrity_hash(&raw, hash).map_err(|message| ManifestError::Invalid {
            path: path.to_path_buf(),
            message,
        })?;
    }
    let skill = parsed.skill.unwrap_or_default();
    let id = skill.id.unwrap_or_default();
    if id.is_empty() {
        return Err(ManifestError::Invalid {
            path: path.to_path_buf(),
            message: "skill.id requis".into(),
        });
    }
    let kind = skill.kind.unwrap_or(SkillPluginKind::Subprocess);
    let plugin = match kind {
        SkillPluginKind::Subprocess => {
            let subprocess = parsed.subprocess.unwrap_or_default();
            let command = subprocess.command.unwrap_or_default();
            if command.is_empty() {
                return Err(ManifestError::Invalid {
                    path: path.to_path_buf(),
                    message: "subprocess.command requis".into(),
                });
            }
            SkillPluginConfig::Subprocess(SubprocessPluginConfig {
                command,
                args: subprocess.args.unwrap_or_default(),
                stdin_json: subprocess.stdin_json.unwrap_or(false),
                timeout_secs: subprocess.timeout_secs.unwrap_or(30),
            })
        }
        SkillPluginKind::Native => {
            let native = parsed.native.unwrap_or_default();
            let library = native.library.unwrap_or_default();
            if library.is_empty() {
                return Err(ManifestError::Invalid {
                    path: path.to_path_buf(),
                    message: "native.library requis".into(),
                });
            }
            SkillPluginConfig::Native(NativePluginConfig { library })
        }
    };
    Ok(SkillManifest {
        id: id.clone(),
        name: skill.name.unwrap_or_else(|| id.clone()),
        description: skill
            .description
            .unwrap_or_else(|| "Skill plugin hub".into()),
        version: skill.version.unwrap_or_else(|| "0.1.0".into()),
        kind,
        enabled: skill.enabled.unwrap_or(true),
        plugin,
        root: path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
    })
}

/// Calcule l'empreinte BLAKE3 d'un manifeste (ligne `integrity_hash` exclue).
#[must_use]
pub fn compute_integrity_hash(raw: &str) -> String {
    let canonical = strip_integrity_line(raw);
    let mut hasher = Hasher::new();
    hasher.update(canonical.as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// Vérifie l'empreinte BLAKE3 d'un manifeste brut.
///
/// # Errors
///
/// Retourne un message d'erreur si le hash ne correspond pas.
pub fn verify_integrity_hash(raw: &str, expected: &str) -> Result<(), String> {
    let normalized = expected.trim().trim_start_matches("blake3:");
    let actual = compute_integrity_hash(raw);
    if constant_time_eq(actual.as_bytes(), normalized.as_bytes()) {
        Ok(())
    } else {
        Err(format!(
            "integrity_hash invalide (attendu {normalized}, obtenu {actual})"
        ))
    }
}

fn strip_integrity_line(raw: &str) -> String {
    raw.lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with("integrity_hash")
                && !trimmed.starts_with("integrity-hash")
        })
        .collect::<Vec<_>>()
        .join("\n")
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

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SkillManifestToml {
    skill: Option<SkillSectionToml>,
    subprocess: Option<SubprocessSectionToml>,
    native: Option<NativeSectionToml>,
}

#[derive(Debug, Default, Deserialize)]
struct SkillSectionToml {
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    version: Option<String>,
    kind: Option<SkillPluginKind>,
    enabled: Option<bool>,
    integrity_hash: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct SubprocessSectionToml {
    command: Option<String>,
    args: Option<Vec<String>>,
    stdin_json: Option<bool>,
    timeout_secs: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
struct NativeSectionToml {
    library: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn loads_subprocess_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("skill.toml");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(
            file,
            r#"
[skill]
id = "pong"
description = "Plugin pong"
version = "0.2.0"

[subprocess]
command = "echo"
args = ["pong"]
stdin_json = false
timeout_secs = 5
"#
        )
        .unwrap();

        let manifest = load_manifest(&path).unwrap();
        assert_eq!(manifest.id, "pong");
        assert_eq!(manifest.kind, SkillPluginKind::Subprocess);
    }

    #[test]
    fn integrity_hash_roundtrip() {
        let unsigned = "[skill]\nid = \"secure\"\ndescription = \"test\"\n\n[subprocess]\ncommand = \"echo\"\nargs = [\"ok\"]\n";
        let hash = compute_integrity_hash(unsigned);
        let signed = format!(
            "[skill]\nid = \"secure\"\ndescription = \"test\"\nintegrity_hash = \"{hash}\"\n\n[subprocess]\ncommand = \"echo\"\nargs = [\"ok\"]\n"
        );
        verify_integrity_hash(&signed, &hash).expect("hash valide");
    }

    #[test]
    fn loads_native_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("skill.toml");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(
            file,
            r#"
[skill]
id = "pong-native"
kind = "native"

[native]
library = "pong_native.dll"
"#
        )
        .unwrap();

        let manifest = load_manifest(&path).unwrap();
        assert_eq!(manifest.kind, SkillPluginKind::Native);
        match manifest.plugin {
            SkillPluginConfig::Native(cfg) => assert_eq!(cfg.library, "pong_native.dll"),
            _ => panic!("kind natif attendu"),
        }
    }
}