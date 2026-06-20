use std::path::{Path, PathBuf};

use cortex::SimilarityThresholds;
use serde::Deserialize;
use thiserror::Error;

/// Configuration des providers IA (primary + fallbacks ordonnés).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvidersConfig {
    /// Provider LLM principal (`xai`, `ollama`, …).
    pub primary_llm: String,
    /// Providers LLM de repli, dans l'ordre de tentative.
    pub fallback_llm: Vec<String>,
    /// Provider d'embeddings principal.
    pub primary_embedding: String,
    /// Providers d'embeddings de repli.
    pub fallback_embedding: Vec<String>,
}

impl Default for ProvidersConfig {
    fn default() -> Self {
        Self {
            primary_llm: "xai".into(),
            fallback_llm: vec!["ollama".into()],
            primary_embedding: "ollama".into(),
            fallback_embedding: vec![],
        }
    }
}

/// Configuration du vector store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorStoreConfig {
    /// Type de store : `memory` (tests) ou `lancedb` (production).
    pub store_type: String,
    /// Chemin `LanceDB` (relatif au workspace si non absolu).
    pub path: PathBuf,
    /// Dimension des vecteurs (ex. 768 pour `nomic-embed-text`).
    pub embedding_dimension: usize,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            store_type: "memory".into(),
            path: PathBuf::from(".orchestrateur/lancedb"),
            embedding_dimension: 768,
        }
    }
}

/// Configuration xAI / Grok.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XaiConfig {
    /// Variable d'environnement contenant la clé API.
    pub api_key_env: String,
    /// Modèle chat (Structured Outputs).
    pub model: String,
    /// Timeout HTTP en secondes.
    pub timeout_secs: u64,
    /// Nombre maximal de tentatives (retry).
    pub max_retries: u32,
}

impl Default for XaiConfig {
    fn default() -> Self {
        Self {
            api_key_env: "XAI_API_KEY".into(),
            model: "grok-3-latest".into(),
            timeout_secs: 30,
            max_retries: 2,
        }
    }
}

/// Configuration Ollama (embeddings + chat fallback).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OllamaConfig {
    /// URL de base Ollama.
    pub url: String,
    /// Modèle d'embeddings.
    pub embedding_model: String,
    /// Modèle de chat.
    pub chat_model: String,
    /// Timeout HTTP en secondes.
    pub timeout_secs: u64,
    /// Nombre maximal de tentatives.
    pub max_retries: u32,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:11434".into(),
            embedding_model: "qwen3-embedding:8b".into(),
            chat_model: "qwen3:8b".into(),
            timeout_secs: 60,
            max_retries: 2,
        }
    }
}

/// Configuration couche 3 — garde comportemental.
#[derive(Debug, Clone, PartialEq)]
pub struct BehavioralConfig {
    /// Active le rate limiting comportemental.
    pub enabled: bool,
    /// Assimilations max par fenêtre.
    pub max_assimilations_per_minute: u32,
    /// Recherches max par fenêtre.
    pub max_searches_per_minute: u32,
    /// Recherches identiques max par fenêtre.
    pub max_repetitive_searches: u32,
    /// Durée de la fenêtre glissante (secondes).
    pub window_secs: u64,
    /// Seuil de blocage du score d'anomalie.
    pub anomaly_block_threshold: f32,
}

impl Default for BehavioralConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_assimilations_per_minute: 60,
            max_searches_per_minute: 120,
            max_repetitive_searches: 15,
            window_secs: 60,
            anomaly_block_threshold: 80.0,
        }
    }
}

/// Configuration couche 2 — intégrité et honeypots.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct IntegrityConfig {
    /// Active la couche intégrité.
    pub enabled: bool,
    /// Vérifie l'empreinte BLAKE3 de `orchestrator.toml`.
    pub verify_config_hash: bool,
    /// Crée le manifeste s'il est absent (trust-on-first-use).
    pub bootstrap_on_missing: bool,
    /// Mode dégradé si manifeste absent.
    pub require_manifest: bool,
    /// Plante des mémoires canari au démarrage.
    pub seed_honeypots: bool,
    /// Nombre de canaris à planter.
    pub honeypot_count: usize,
}

impl Default for IntegrityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            verify_config_hash: true,
            bootstrap_on_missing: true,
            require_manifest: false,
            seed_honeypots: false,
            honeypot_count: 3,
        }
    }
}

/// Configuration couche 4 — audit tamper-evident.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditConfig {
    /// Active le journal d'audit chaîné.
    pub enabled: bool,
    /// Chemin relatif au workspace.
    pub path: PathBuf,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: PathBuf::from("logs/audit.jsonl"),
        }
    }
}

/// Configuration de la couche sécurité (défense en profondeur).
#[derive(Debug, Clone, PartialEq)]
pub struct SecurityConfig {
    /// Active la validation adversariale des [`MemoryDraft`] (couche 1).
    pub enabled: bool,
    /// Longueur maximale du contenu Markdown.
    pub max_content_length: usize,
    /// Longueur maximale du titre.
    pub max_title_length: usize,
    /// Nombre maximal de tags.
    pub max_tags: usize,
    /// Nombre maximal de backlinks candidats issus du LLM.
    pub max_backlinks: usize,
    /// Détecte les patterns d'injection / poisoning connus.
    pub detect_injection_patterns: bool,
    /// Couche 3 — comportemental.
    pub behavioral: BehavioralConfig,
    /// Couche 2 — intégrité.
    pub integrity: IntegrityConfig,
    /// Couche 4 — audit.
    pub audit: AuditConfig,
    /// Profil actif (`ai_assisted`, `strict`, `expert`, …).
    pub profile: Option<crate::security::SecurityProfile>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_content_length: 64_000,
            max_title_length: 512,
            max_tags: 32,
            max_backlinks: 20,
            detect_injection_patterns: true,
            behavioral: BehavioralConfig::default(),
            integrity: IntegrityConfig::default(),
            audit: AuditConfig::default(),
            profile: None,
        }
    }
}

/// Configuration applicative de l'orchestrateur.
#[derive(Debug, Clone, PartialEq)]
pub struct OrchestratorConfig {
    /// Racine du workspace utilisateur (mémoires, config, vecteurs).
    pub workspace_root: PathBuf,
    /// Seuils pour le calcul des backlinks sémantiques.
    pub similarity_thresholds: SimilarityThresholds,
    /// Dimension des embeddings (mocks in-memory ; alignée sur `LanceDB` en prod).
    pub embedding_dim: usize,
    /// Routing multi-provider.
    pub providers: ProvidersConfig,
    /// Vector store.
    pub vector_store: VectorStoreConfig,
    /// Section xAI.
    pub xai: XaiConfig,
    /// Section Ollama.
    pub ollama: OllamaConfig,
    /// Couche sécurité adversariale (validation des brouillons LLM).
    pub security: SecurityConfig,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        let vector_store = VectorStoreConfig::default();
        Self {
            workspace_root: PathBuf::from("workspace"),
            similarity_thresholds: SimilarityThresholds::default(),
            embedding_dim: vector_store.embedding_dimension,
            providers: ProvidersConfig::default(),
            vector_store,
            xai: XaiConfig::default(),
            ollama: OllamaConfig::default(),
            security: SecurityConfig::default(),
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

    /// Chemin résolu du vector store `LanceDB`.
    #[must_use]
    pub fn lancedb_path(&self) -> PathBuf {
        if self.vector_store.path.is_absolute() {
            self.vector_store.path.clone()
        } else {
            self.workspace_root.join(&self.vector_store.path)
        }
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
            ..Self::default()
        };
        config.embedding_dim = config.vector_store.embedding_dimension;
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
            if let Some(root) = ws.path.or(ws.root) {
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
        if let Some(p) = settings.providers {
            if let Some(v) = p.primary_llm {
                self.providers.primary_llm = v;
            }
            if let Some(v) = p.fallback_llm {
                self.providers.fallback_llm = v;
            }
            if let Some(v) = p.primary_embedding {
                self.providers.primary_embedding = v;
            }
            if let Some(v) = p.fallback_embedding {
                self.providers.fallback_embedding = v;
            }
        }
        if let Some(vs) = settings.vector_store {
            if let Some(t) = vs.r#type {
                self.vector_store.store_type = t;
            }
            if let Some(p) = vs.path {
                self.vector_store.path = PathBuf::from(p);
            }
            if let Some(d) = vs.embedding_dimension {
                self.vector_store.embedding_dimension = d;
                self.embedding_dim = d;
            }
        }
        if let Some(ld) = settings.lancedb {
            self.vector_store.store_type = "lancedb".into();
            if let Some(p) = ld.path {
                self.vector_store.path = PathBuf::from(p);
            }
            if let Some(d) = ld.embedding_dimension {
                self.vector_store.embedding_dimension = d;
                self.embedding_dim = d;
            }
        }
        if let Some(x) = settings.xai {
            if let Some(v) = x.api_key_env {
                self.xai.api_key_env = v;
            }
            if let Some(v) = x.model {
                self.xai.model = v;
            }
            if let Some(v) = x.timeout_secs {
                self.xai.timeout_secs = v;
            }
            if let Some(v) = x.max_retries {
                self.xai.max_retries = v;
            }
        }
        if let Some(o) = settings.ollama {
            if let Some(v) = o.url {
                self.ollama.url = v;
            }
            if let Some(v) = o.embedding_model {
                self.ollama.embedding_model = v;
            }
            if let Some(v) = o.chat_model {
                self.ollama.chat_model = v;
            }
            if let Some(v) = o.timeout_secs {
                self.ollama.timeout_secs = v;
            }
            if let Some(v) = o.max_retries {
                self.ollama.max_retries = v;
            }
        }
        if let Some(s) = settings.security {
            merge_security(&mut self.security, s);
        }
    }
}

fn merge_security(target: &mut SecurityConfig, section: SecuritySection) {
    if let Some(ref name) = section.profile {
        if let Some(profile) = crate::security::SecurityProfile::parse(name) {
            profile.apply(target);
        } else {
            tracing::warn!(profile = %name, "profil sécurité inconnu — ignoré");
        }
    }
    if let Some(v) = section.enabled {
        target.enabled = v;
    }
    if let Some(v) = section.max_content_length {
        target.max_content_length = v;
    }
    if let Some(v) = section.max_title_length {
        target.max_title_length = v;
    }
    if let Some(v) = section.max_tags {
        target.max_tags = v;
    }
    if let Some(v) = section.max_backlinks {
        target.max_backlinks = v;
    }
    if let Some(v) = section.detect_injection_patterns {
        target.detect_injection_patterns = v;
    }
    if let Some(b) = section.behavioral {
        if let Some(v) = b.enabled {
            target.behavioral.enabled = v;
        }
        if let Some(v) = b.max_assimilations_per_minute {
            target.behavioral.max_assimilations_per_minute = v;
        }
        if let Some(v) = b.max_searches_per_minute {
            target.behavioral.max_searches_per_minute = v;
        }
        if let Some(v) = b.max_repetitive_searches {
            target.behavioral.max_repetitive_searches = v;
        }
        if let Some(v) = b.window_secs {
            target.behavioral.window_secs = v;
        }
        if let Some(v) = b.anomaly_block_threshold {
            target.behavioral.anomaly_block_threshold = v;
        }
    }
    if let Some(i) = section.integrity {
        if let Some(v) = i.enabled {
            target.integrity.enabled = v;
        }
        if let Some(v) = i.verify_config_hash {
            target.integrity.verify_config_hash = v;
        }
        if let Some(v) = i.bootstrap_on_missing {
            target.integrity.bootstrap_on_missing = v;
        }
        if let Some(v) = i.require_manifest {
            target.integrity.require_manifest = v;
        }
        if let Some(v) = i.seed_honeypots {
            target.integrity.seed_honeypots = v;
        }
        if let Some(v) = i.honeypot_count {
            target.integrity.honeypot_count = v;
        }
    }
    if let Some(a) = section.audit {
        if let Some(v) = a.enabled {
            target.audit.enabled = v;
        }
        if let Some(v) = a.path {
            target.audit.path = PathBuf::from(v);
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
    providers: Option<ProvidersSection>,
    vector_store: Option<VectorStoreSection>,
    lancedb: Option<LancedbSection>,
    xai: Option<XaiSection>,
    ollama: Option<OllamaSection>,
    security: Option<SecuritySection>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceSection {
    path: Option<String>,
    root: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BacklinksSection {
    semantic_min: Option<f32>,
    max_links: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct ProvidersSection {
    primary_llm: Option<String>,
    fallback_llm: Option<Vec<String>>,
    primary_embedding: Option<String>,
    fallback_embedding: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct VectorStoreSection {
    r#type: Option<String>,
    path: Option<String>,
    embedding_dimension: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct LancedbSection {
    path: Option<String>,
    embedding_dimension: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct XaiSection {
    api_key_env: Option<String>,
    model: Option<String>,
    timeout_secs: Option<u64>,
    max_retries: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OllamaSection {
    url: Option<String>,
    embedding_model: Option<String>,
    chat_model: Option<String>,
    timeout_secs: Option<u64>,
    max_retries: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct SecuritySection {
    profile: Option<String>,
    enabled: Option<bool>,
    max_content_length: Option<usize>,
    max_title_length: Option<usize>,
    max_tags: Option<usize>,
    max_backlinks: Option<usize>,
    detect_injection_patterns: Option<bool>,
    behavioral: Option<BehavioralSection>,
    integrity: Option<IntegritySection>,
    audit: Option<AuditSection>,
}

#[derive(Debug, Deserialize)]
struct BehavioralSection {
    enabled: Option<bool>,
    max_assimilations_per_minute: Option<u32>,
    max_searches_per_minute: Option<u32>,
    max_repetitive_searches: Option<u32>,
    window_secs: Option<u64>,
    anomaly_block_threshold: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct IntegritySection {
    enabled: Option<bool>,
    verify_config_hash: Option<bool>,
    bootstrap_on_missing: Option<bool>,
    require_manifest: Option<bool>,
    seed_honeypots: Option<bool>,
    honeypot_count: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct AuditSection {
    enabled: Option<bool>,
    path: Option<String>,
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
        assert_eq!(cfg.embedding_dim, 768);
        assert_eq!(cfg.providers.primary_llm, "xai");
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
    fn loads_providers_and_lancedb_from_toml() {
        let dir = tempfile::tempdir().unwrap();
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&config_dir).unwrap();
        let toml_path = config_dir.join("orchestrator.toml");
        let mut file = std::fs::File::create(&toml_path).unwrap();
        writeln!(
            file,
            r#"
[providers]
primary_llm = "ollama"
fallback_llm = ["xai"]
primary_embedding = "ollama"

[lancedb]
path = ".orchestrateur/lancedb"
embedding_dimension = 384

[ollama]
url = "http://127.0.0.1:11434"
embedding_model = "nomic-embed-text"
"#
        )
        .unwrap();

        let cfg = OrchestratorConfig::load_workspace(dir.path()).unwrap();
        assert_eq!(cfg.providers.primary_llm, "ollama");
        assert_eq!(cfg.vector_store.store_type, "lancedb");
        assert_eq!(cfg.embedding_dim, 384);
        assert_eq!(cfg.ollama.embedding_model, "nomic-embed-text");
    }

    #[test]
    fn loads_ai_assisted_profile_from_toml() {
        let dir = tempfile::tempdir().unwrap();
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&config_dir).unwrap();
        let toml_path = config_dir.join("orchestrator.toml");
        let mut file = std::fs::File::create(&toml_path).unwrap();
        writeln!(
            file,
            r#"
[security]
profile = "ai_assisted"
"#
        )
        .unwrap();

        let cfg = OrchestratorConfig::load_workspace(dir.path()).unwrap();
        assert_eq!(
            cfg.security.profile,
            Some(crate::security::SecurityProfile::AiAssisted)
        );
        assert_eq!(cfg.security.behavioral.max_assimilations_per_minute, 300);
        assert_eq!(cfg.security.behavioral.max_repetitive_searches, 80);
    }

    #[test]
    fn profile_explicit_override_wins() {
        let dir = tempfile::tempdir().unwrap();
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&config_dir).unwrap();
        let toml_path = config_dir.join("orchestrator.toml");
        let mut file = std::fs::File::create(&toml_path).unwrap();
        writeln!(
            file,
            r#"
[security]
profile = "ai_assisted"

[security.behavioral]
max_assimilations_per_minute = 500
"#
        )
        .unwrap();

        let cfg = OrchestratorConfig::load_workspace(dir.path()).unwrap();
        assert_eq!(cfg.security.behavioral.max_assimilations_per_minute, 500);
        assert_eq!(cfg.security.behavioral.max_searches_per_minute, 600);
    }

    #[test]
    fn loads_security_from_toml() {
        let dir = tempfile::tempdir().unwrap();
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&config_dir).unwrap();
        let toml_path = config_dir.join("orchestrator.toml");
        let mut file = std::fs::File::create(&toml_path).unwrap();
        writeln!(
            file,
            r#"
[security]
enabled = true
max_content_length = 10000
max_backlinks = 5
detect_injection_patterns = false
"#
        )
        .unwrap();

        let cfg = OrchestratorConfig::load_workspace(dir.path()).unwrap();
        assert!(cfg.security.enabled);
        assert_eq!(cfg.security.max_content_length, 10_000);
        assert_eq!(cfg.security.max_backlinks, 5);
        assert!(!cfg.security.detect_injection_patterns);
    }

    #[test]
    fn missing_toml_uses_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = OrchestratorConfig::load_workspace(dir.path()).unwrap();
        assert_eq!(cfg.similarity_thresholds.max_links, 10);
        assert_eq!(cfg.vector_store.store_type, "memory");
    }
}
