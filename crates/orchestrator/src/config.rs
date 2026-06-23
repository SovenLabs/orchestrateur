use std::collections::HashMap;
use std::path::{Path, PathBuf};

use cortex::{MemoryDraftValidatorConfig, SimilarityThresholds};
use serde::Deserialize;
use thiserror::Error;

use crate::providers::ProviderProfileSection;
use crate::providers::ProviderProfiles;

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
            primary_llm: "ollama".into(),
            fallback_llm: vec![],
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
            store_type: "lancedb".into(),
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
            model: "grok-4.3".into(),
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
    /// Bloque les providers LLM cloud (profil `local_only`).
    pub block_cloud_llm: bool,
    /// Scan les secrets dans le texte avant chat/assimilate.
    pub scan_secrets_before_llm: bool,
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
            block_cloud_llm: false,
            scan_secrets_before_llm: false,
        }
    }
}

impl SecurityConfig {
    /// Projette la configuration sécurité applicative vers le validateur domaine [`cortex`].
    #[must_use]
    pub fn validator_config(&self) -> MemoryDraftValidatorConfig {
        MemoryDraftValidatorConfig {
            min_content_length: 1,
            max_content_length: self.max_content_length,
            max_title_length: self.max_title_length,
            max_tags: self.max_tags,
            max_backlinks: self.max_backlinks,
            detect_injection_patterns: self.detect_injection_patterns,
            blocking_risk_threshold: 0.85,
        }
    }
}

/// Configuration d'un canal gateway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayChannelConfig {
    /// Active le canal au démarrage du gateway.
    pub enabled: bool,
    /// Variable d'environnement du token / secret du canal.
    pub token_env: String,
    /// URL HTTP de polling entrant (canaux stub Phase 14).
    pub poll_url: Option<String>,
    /// Intervalle entre deux polls HTTP (secondes).
    pub poll_interval_secs: u64,
}

impl GatewayChannelConfig {
    /// Canal désactivé par défaut.
    #[must_use]
    pub fn disabled(token_env: impl Into<String>) -> Self {
        Self {
            enabled: false,
            token_env: token_env.into(),
            poll_url: None,
            poll_interval_secs: 30,
        }
    }
}

/// Configuration du daemon WebSocket local (Territoire Graphique — Option B).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonConfig {
    /// Active le daemon (`daemon run` refuse si false).
    pub enabled: bool,
    /// Adresse de liaison.
    pub bind: String,
    /// Port d'écoute du daemon local (défaut 28790).
    pub port: u16,
    /// Variable d'environnement du token d'authentification WS.
    pub token_env: String,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bind: "127.0.0.1".into(),
            port: 28_790,
            token_env: "ORCHESTRATEUR_DAEMON_TOKEN".into(),
        }
    }
}

/// Configuration du gateway WebSocket Phase 8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayConfig {
    /// Active le gateway (sinon `gateway run` refuse de démarrer).
    pub enabled: bool,
    /// Adresse de liaison.
    pub bind: String,
    /// Port d'écoute du gateway local (défaut 28789).
    pub port: u16,
    /// Variable d'environnement du token d'authentification WS.
    pub token_env: String,
    /// Canal webhook HTTP entrant.
    pub webhook: GatewayChannelConfig,
    /// Canal Telegram (long-polling si token présent).
    pub telegram: GatewayChannelConfig,
    /// Canal Discord (webhook sortant si token présent).
    pub discord: GatewayChannelConfig,
    /// Canal Slack (stub API si token présent).
    pub slack: GatewayChannelConfig,
    /// Canaux additionnels Phase 10 (whatsapp, matrix, …).
    pub extra_channels: HashMap<String, GatewayChannelConfig>,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bind: "127.0.0.1".into(),
            port: 28_789,
            token_env: "ORCHESTRATEUR_GATEWAY_TOKEN".into(),
            webhook: GatewayChannelConfig {
                enabled: true,
                token_env: "ORCHESTRATEUR_WEBHOOK_SECRET".into(),
                poll_url: None,
                poll_interval_secs: 30,
            },
            telegram: GatewayChannelConfig {
                enabled: true,
                token_env: "TELEGRAM_BOT_TOKEN".into(),
                poll_url: None,
                poll_interval_secs: 30,
            },
            discord: GatewayChannelConfig {
                enabled: true,
                token_env: "DISCORD_BOT_TOKEN".into(),
                poll_url: None,
                poll_interval_secs: 30,
            },
            slack: GatewayChannelConfig {
                enabled: true,
                token_env: "SLACK_BOT_TOKEN".into(),
                poll_url: None,
                poll_interval_secs: 30,
            },
            extra_channels: HashMap::new(),
        }
    }
}

/// Configuration d'un serveur MCP (stdio).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpServerConfig {
    /// Nom logique du serveur.
    pub name: String,
    /// Commande à exécuter.
    pub command: String,
    /// Arguments de la commande.
    pub args: Vec<String>,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            name: "default".into(),
            command: String::new(),
            args: Vec::new(),
        }
    }
}

/// Configuration agent Phase 10 (persistée TOML `[agent]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSettingsConfig {
    /// Nombre maximal d'itérations outil par tour.
    pub max_tool_iterations: usize,
    /// Injecte le contexte graphe dans le prompt.
    pub graph_context_enabled: bool,
    /// Nombre de hubs dans le contexte graphe.
    pub graph_hub_limit: usize,
    /// Recherche mémoire proactive avant LLM.
    pub proactive_memory_search: bool,
    /// Limite résultats recherche proactive.
    pub proactive_search_limit: usize,
    /// Auto-assimilation systématique de chaque tour (différenciateur Phase 10).
    pub auto_assimilate_turn: bool,
    /// Historique maximal envoyé au LLM.
    pub max_history_turns: usize,
    /// Profil de capacités actif (`agent`, `memory`, `full`, …).
    pub active_capability_profile: String,
    /// Outils agent `skill_list` / `skill_execute` (Phase 12).
    pub skill_tools_enabled: bool,
    /// Injecte le catalogue skills + suggestions dans le prompt agent (Phase 13).
    pub skill_auto_suggest: bool,
    /// Exécute automatiquement la skill la mieux notée avant le LLM (Phase 14).
    pub skill_auto_execute: bool,
    /// Score minimal pour déclencher l'auto-exécution (Phase 14).
    pub skill_auto_execute_threshold: u32,
    /// Prétraitement bidirectionnel des messages avant construction du contexte (PR-6).
    pub message_preprocess: bool,
    /// Seuil minimal (caractères) en dessous duquel un message est enrichi.
    pub enrichment_min_chars: usize,
    /// Seuil maximal (caractères) au-delà duquel un message est compressé.
    pub compression_max_chars: usize,
    /// Préserve les entités nommées via ancrage Cortex lors de la compression.
    pub compression_preserve_entities: bool,
}

impl Default for AgentSettingsConfig {
    fn default() -> Self {
        Self {
            max_tool_iterations: 3,
            graph_context_enabled: true,
            graph_hub_limit: 5,
            proactive_memory_search: true,
            proactive_search_limit: 5,
            auto_assimilate_turn: true,
            max_history_turns: 20,
            active_capability_profile: "agent".into(),
            skill_tools_enabled: true,
            skill_auto_suggest: true,
            skill_auto_execute: false,
            skill_auto_execute_threshold: 10,
            message_preprocess: true,
            enrichment_min_chars: 40,
            compression_max_chars: 8000,
            compression_preserve_entities: true,
        }
    }
}

/// Entrée inline d'un plugin subprocess (Phase 11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillsHubEntryConfig {
    /// Identifiant stable de la skill.
    pub id: String,
    /// Description affichée.
    pub description: String,
    /// Active l'entrée au chargement.
    pub enabled: bool,
    /// Commande exécutable.
    pub command: String,
    /// Arguments de la commande.
    pub args: Vec<String>,
    /// Envoie le [`SkillContext`] en JSON sur stdin.
    pub stdin_json: bool,
    /// Timeout subprocess en secondes.
    pub timeout_secs: u64,
}

impl Default for SkillsHubEntryConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            description: String::new(),
            enabled: true,
            command: String::new(),
            args: Vec::new(),
            stdin_json: false,
            timeout_secs: 30,
        }
    }
}

/// Configuration du hub de skills dynamiques (Phase 11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillsHubConfig {
    /// Active le scan et le chargement du hub.
    pub enabled: bool,
    /// Répertoire relatif au workspace (`skills/` par défaut).
    pub directory: String,
    /// Charge automatiquement les plugins au démarrage de la facade.
    pub auto_load: bool,
    /// Plugins déclarés inline dans `orchestrator.toml`.
    pub entries: Vec<SkillsHubEntryConfig>,
    /// Active le catalogue marketplace (Phase 13).
    pub marketplace_enabled: bool,
    /// Chemin relatif au workspace du `catalog.json` local.
    pub marketplace_catalog: String,
    /// URL catalogue distant optionnel (feature `skills-marketplace`).
    pub marketplace_url: Option<String>,
    /// Exige un `catalog_hash` BLAKE3 valide au chargement (Phase 14).
    pub marketplace_require_signature: bool,
}

impl Default for SkillsHubConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            directory: "skills".into(),
            auto_load: true,
            entries: Vec::new(),
            marketplace_enabled: true,
            marketplace_catalog: "skills/marketplace/catalog.json".into(),
            marketplace_url: None,
            marketplace_require_signature: false,
        }
    }
}

/// Configuration du watcher de sessions Markdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatcherConfig {
    /// Active le watcher au démarrage daemon / CLI `watch`.
    pub enabled: bool,
    /// Répertoires relatifs au workspace à surveiller (récursif `*.md`).
    pub watch_dirs: Vec<String>,
    /// Délai de stabilité fichier avant traitement (secondes).
    pub debounce_secs: u64,
    /// Taille minimale du contenu session (caractères).
    pub min_content_chars: usize,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            watch_dirs: vec![".orchestrateur/sessions".into()],
            debounce_secs: 8,
            min_content_chars: 120,
        }
    }
}

/// Configuration mémoire opérationnelle (insights, dédup).
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryConfig {
    /// Seuil Jaccard pour considérer un brouillon comme doublon (0.0–1.0).
    pub dedup_jaccard_threshold: f32,
    /// Nombre de souvenirs liés injectés avant extraction LLM.
    pub insight_related_search_limit: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            dedup_jaccard_threshold: 0.6,
            insight_related_search_limit: 3,
        }
    }
}

/// Configuration client MCP Phase 9.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpConfig {
    /// Active le client MCP et les outils `mcp_*`.
    pub enabled: bool,
    /// Serveurs MCP à connecter au démarrage.
    pub servers: Vec<McpServerConfig>,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            servers: Vec::new(),
        }
    }
}

/// Édition TOML pour onboard / CLI harness.
pub mod editor;

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
    /// Daemon WebSocket local pour clients visuels (Phase 14 bis).
    pub daemon: DaemonConfig,
    /// Gateway WebSocket + canaux (Phase 8).
    pub gateway: GatewayConfig,
    /// Profils surchargeables par provider (Phase 9).
    pub provider_profiles: ProviderProfiles,
    /// Client MCP (Phase 9).
    pub mcp: McpConfig,
    /// Configuration agent (Phase 10).
    pub agent: AgentSettingsConfig,
    /// Insights, dédup et extraction mémoire.
    pub memory: MemoryConfig,
    /// Watcher sessions → brouillons insight.
    pub watcher: WatcherConfig,
    /// Hub skills + plugins dynamiques (Phase 11).
    pub skills_hub: SkillsHubConfig,
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
            daemon: DaemonConfig::default(),
            gateway: GatewayConfig::default(),
            provider_profiles: ProviderProfiles::default(),
            mcp: McpConfig::default(),
            agent: AgentSettingsConfig::default(),
            memory: MemoryConfig::default(),
            watcher: WatcherConfig::default(),
            skills_hub: SkillsHubConfig::default(),
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

    /// Répertoire du hub de skills (`workspace/skills` par défaut).
    #[must_use]
    pub fn skills_hub_dir(&self) -> PathBuf {
        self.workspace_root.join(&self.skills_hub.directory)
    }

    /// Chemin du catalogue marketplace local.
    #[must_use]
    pub fn marketplace_catalog_path(&self) -> PathBuf {
        self.workspace_root.join(&self.skills_hub.marketplace_catalog)
    }

    /// Répertoire par défaut des fichiers session Markdown surveillés.
    #[must_use]
    pub fn sessions_watch_dir(&self) -> PathBuf {
        self.workspace_root.join(".orchestrateur").join("sessions")
    }

    /// Répertoire des brouillons en attente de publication.
    #[must_use]
    pub fn drafts_dir(&self) -> PathBuf {
        self.workspace_root.join(".orchestrateur").join("drafts")
    }

    /// Chemin de la base SQLite des sessions agent.
    #[must_use]
    pub fn sessions_db_path(&self) -> PathBuf {
        self.workspace_root
            .join(".orchestrateur")
            .join("sessions.db")
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
            tracing::warn!(
                "[lancedb] est déprécié — utilisez [vector_store] type = \"lancedb\""
            );
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
        if let Some(d) = settings.daemon {
            merge_daemon(&mut self.daemon, d);
        }
        if let Some(g) = settings.gateway {
            merge_gateway(&mut self.gateway, g);
        }
        if let Some(profiles) = settings.provider_profiles {
            for (id, section) in profiles {
                self.provider_profiles.merge_section(id, section);
            }
        }
        if let Some(m) = settings.mcp {
            merge_mcp(&mut self.mcp, m);
        }
        if let Some(a) = settings.agent {
            merge_agent(&mut self.agent, a);
        }
        if let Some(m) = settings.memory {
            merge_memory(&mut self.memory, m);
        }
        if let Some(w) = settings.watcher {
            merge_watcher(&mut self.watcher, w);
        }
        if let Some(h) = settings.skills_hub {
            merge_skills_hub(&mut self.skills_hub, h);
        }
    }
}

fn merge_watcher(target: &mut WatcherConfig, section: WatcherSection) {
    if let Some(v) = section.enabled {
        target.enabled = v;
    }
    if let Some(v) = section.watch_dirs {
        target.watch_dirs = v;
    }
    if let Some(v) = section.debounce_secs {
        target.debounce_secs = v.max(2);
    }
    if let Some(v) = section.min_content_chars {
        target.min_content_chars = v.max(32);
    }
}

fn merge_skills_hub(target: &mut SkillsHubConfig, section: SkillsHubSection) {
    if let Some(v) = section.enabled {
        target.enabled = v;
    }
    if let Some(v) = section.directory {
        target.directory = v;
    }
    if let Some(v) = section.auto_load {
        target.auto_load = v;
    }
    if let Some(entries) = section.entries {
        target.entries = entries
            .into_iter()
            .map(|e| SkillsHubEntryConfig {
                id: e.id.unwrap_or_default(),
                description: e.description.unwrap_or_default(),
                enabled: e.enabled.unwrap_or(true),
                command: e.command.unwrap_or_default(),
                args: e.args.unwrap_or_default(),
                stdin_json: e.stdin_json.unwrap_or(false),
                timeout_secs: e.timeout_secs.unwrap_or(30),
            })
            .filter(|e| !e.id.is_empty() && !e.command.is_empty())
            .collect();
    }
    if let Some(v) = section.marketplace_enabled {
        target.marketplace_enabled = v;
    }
    if let Some(v) = section.marketplace_catalog {
        target.marketplace_catalog = v;
    }
    if let Some(v) = section.marketplace_url {
        target.marketplace_url = Some(v);
    }
    if let Some(v) = section.marketplace_require_signature {
        target.marketplace_require_signature = v;
    }
}

fn merge_memory(target: &mut MemoryConfig, section: MemorySection) {
    if let Some(v) = section.dedup_jaccard_threshold {
        target.dedup_jaccard_threshold = v;
    }
    if let Some(v) = section.insight_related_search_limit {
        target.insight_related_search_limit = v;
    }
}

fn merge_agent(target: &mut AgentSettingsConfig, section: AgentSection) {
    if let Some(v) = section.max_tool_iterations {
        target.max_tool_iterations = v;
    }
    if let Some(v) = section.graph_context_enabled {
        target.graph_context_enabled = v;
    }
    if let Some(v) = section.graph_hub_limit {
        target.graph_hub_limit = v;
    }
    if let Some(v) = section.proactive_memory_search {
        target.proactive_memory_search = v;
    }
    if let Some(v) = section.proactive_search_limit {
        target.proactive_search_limit = v;
    }
    if let Some(v) = section.auto_assimilate_turn {
        target.auto_assimilate_turn = v;
    }
    if let Some(v) = section.max_history_turns {
        target.max_history_turns = v;
    }
    if let Some(v) = section.active_capability_profile {
        target.active_capability_profile = v;
    }
    if let Some(v) = section.skill_tools_enabled {
        target.skill_tools_enabled = v;
    }
    if let Some(v) = section.skill_auto_suggest {
        target.skill_auto_suggest = v;
    }
    if let Some(v) = section.skill_auto_execute {
        target.skill_auto_execute = v;
    }
    if let Some(v) = section.skill_auto_execute_threshold {
        target.skill_auto_execute_threshold = v;
    }
    if let Some(v) = section.message_preprocess {
        target.message_preprocess = v;
    }
    if let Some(v) = section.enrichment_min_chars {
        target.enrichment_min_chars = v;
    }
    if let Some(v) = section.compression_max_chars {
        target.compression_max_chars = v;
    }
    if let Some(v) = section.compression_preserve_entities {
        target.compression_preserve_entities = v;
    }
}

fn merge_mcp(target: &mut McpConfig, section: McpSection) {
    if let Some(v) = section.enabled {
        target.enabled = v;
    }
    if let Some(servers) = section.servers {
        target.servers = servers
            .into_iter()
            .map(|s| McpServerConfig {
                name: s.name.unwrap_or_else(|| "default".into()),
                command: s.command.unwrap_or_default(),
                args: s.args.unwrap_or_default(),
            })
            .filter(|s| !s.command.is_empty())
            .collect();
    }
}

fn merge_daemon(target: &mut DaemonConfig, section: DaemonSection) {
    if let Some(v) = section.enabled {
        target.enabled = v;
    }
    if let Some(v) = section.bind {
        target.bind = v;
    }
    if let Some(v) = section.port {
        target.port = v;
    }
    if let Some(v) = section.token_env {
        target.token_env = v;
    }
}

fn merge_gateway(target: &mut GatewayConfig, section: GatewaySection) {
    if let Some(v) = section.enabled {
        target.enabled = v;
    }
    if let Some(v) = section.bind {
        target.bind = v;
    }
    if let Some(v) = section.port {
        target.port = v;
    }
    if let Some(v) = section.token_env {
        target.token_env = v;
    }
    if let Some(w) = section.webhook {
        merge_gateway_channel(&mut target.webhook, w);
    }
    if let Some(t) = section.telegram {
        merge_gateway_channel(&mut target.telegram, t);
    }
    if let Some(d) = section.discord {
        merge_gateway_channel(&mut target.discord, d);
    }
    if let Some(s) = section.slack {
        merge_gateway_channel(&mut target.slack, s);
    }
    if let Some(channels) = section.channels {
        for (id, cfg) in channels {
            let entry = target
                .extra_channels
                .entry(id)
                .or_insert_with(|| GatewayChannelConfig::disabled(String::new()));
            merge_gateway_channel(entry, cfg);
        }
    }
}

fn merge_gateway_channel(target: &mut GatewayChannelConfig, section: GatewayChannelSection) {
    if let Some(v) = section.enabled {
        target.enabled = v;
    }
    if let Some(v) = section.token_env {
        target.token_env = v;
    }
    if let Some(v) = section.poll_url {
        target.poll_url = if v.is_empty() { None } else { Some(v) };
    }
    if let Some(v) = section.poll_interval_secs {
        target.poll_interval_secs = v.max(5);
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
    if let Some(v) = section.block_cloud_llm {
        target.block_cloud_llm = v;
    }
    if let Some(v) = section.scan_secrets_before_llm {
        target.scan_secrets_before_llm = v;
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
    daemon: Option<DaemonSection>,
    gateway: Option<GatewaySection>,
    provider_profiles: Option<HashMap<String, ProviderProfileSection>>,
    mcp: Option<McpSection>,
    agent: Option<AgentSection>,
    memory: Option<MemorySection>,
    watcher: Option<WatcherSection>,
    skills_hub: Option<SkillsHubSection>,
}

#[derive(Debug, Deserialize)]
struct WatcherSection {
    enabled: Option<bool>,
    watch_dirs: Option<Vec<String>>,
    debounce_secs: Option<u64>,
    min_content_chars: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct MemorySection {
    dedup_jaccard_threshold: Option<f32>,
    insight_related_search_limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct SkillsHubSection {
    enabled: Option<bool>,
    directory: Option<String>,
    auto_load: Option<bool>,
    entries: Option<Vec<SkillsHubEntryToml>>,
    marketplace_enabled: Option<bool>,
    marketplace_catalog: Option<String>,
    marketplace_url: Option<String>,
    marketplace_require_signature: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct SkillsHubEntryToml {
    id: Option<String>,
    description: Option<String>,
    enabled: Option<bool>,
    command: Option<String>,
    args: Option<Vec<String>>,
    stdin_json: Option<bool>,
    timeout_secs: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct AgentSection {
    max_tool_iterations: Option<usize>,
    graph_context_enabled: Option<bool>,
    graph_hub_limit: Option<usize>,
    proactive_memory_search: Option<bool>,
    proactive_search_limit: Option<usize>,
    auto_assimilate_turn: Option<bool>,
    max_history_turns: Option<usize>,
    #[serde(alias = "active_toolset")]
    active_capability_profile: Option<String>,
    skill_tools_enabled: Option<bool>,
    skill_auto_suggest: Option<bool>,
    skill_auto_execute: Option<bool>,
    skill_auto_execute_threshold: Option<u32>,
    message_preprocess: Option<bool>,
    enrichment_min_chars: Option<usize>,
    compression_max_chars: Option<usize>,
    compression_preserve_entities: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct McpSection {
    enabled: Option<bool>,
    servers: Option<Vec<McpServerToml>>,
}

#[derive(Debug, Deserialize)]
struct McpServerToml {
    name: Option<String>,
    command: Option<String>,
    args: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct DaemonSection {
    enabled: Option<bool>,
    bind: Option<String>,
    port: Option<u16>,
    token_env: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GatewaySection {
    enabled: Option<bool>,
    bind: Option<String>,
    port: Option<u16>,
    token_env: Option<String>,
    webhook: Option<GatewayChannelSection>,
    telegram: Option<GatewayChannelSection>,
    discord: Option<GatewayChannelSection>,
    slack: Option<GatewayChannelSection>,
    channels: Option<HashMap<String, GatewayChannelSection>>,
}

#[derive(Debug, Deserialize)]
struct GatewayChannelSection {
    enabled: Option<bool>,
    token_env: Option<String>,
    poll_url: Option<String>,
    poll_interval_secs: Option<u64>,
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
    block_cloud_llm: Option<bool>,
    scan_secrets_before_llm: Option<bool>,
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
        assert_eq!(cfg.providers.primary_llm, "ollama");
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
    fn loads_provider_profiles_from_toml() {
        let dir = tempfile::tempdir().unwrap();
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&config_dir).unwrap();
        let toml_path = config_dir.join("orchestrator.toml");
        let mut file = std::fs::File::create(&toml_path).unwrap();
        writeln!(
            file,
            r#"
[provider_profiles.openai]
model = "gpt-4o-mini"
api_key_env = "OPENAI_API_KEY"

[provider_profiles.groq]
enabled = false
"#
        )
        .unwrap();

        let cfg = OrchestratorConfig::load_workspace(dir.path()).unwrap();
        let registry = crate::providers::ProviderRegistry::new();
        let openai = registry.llm_descriptor("openai").unwrap();
        let profile = cfg.provider_profiles.resolve("openai", openai);
        assert_eq!(profile.model, "gpt-4o-mini");
        let groq = registry.llm_descriptor("groq").unwrap();
        let groq_profile = cfg.provider_profiles.resolve("groq", groq);
        assert!(!groq_profile.enabled);
    }

    #[test]
    fn agent_defaults_auto_assimilate_turn_true() {
        let cfg = OrchestratorConfig::default();
        assert!(cfg.agent.auto_assimilate_turn);
        assert!(cfg.agent.skill_tools_enabled);
        assert_eq!(cfg.agent.active_capability_profile, "agent");
        assert_eq!(cfg.agent.max_tool_iterations, 3);
        assert!(cfg.agent.message_preprocess);
        assert_eq!(cfg.agent.enrichment_min_chars, 40);
        assert_eq!(cfg.agent.compression_max_chars, 8000);
        assert!(cfg.agent.compression_preserve_entities);
    }

    #[test]
    fn loads_agent_and_extra_channels_from_toml() {
        let dir = tempfile::tempdir().unwrap();
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&config_dir).unwrap();
        let toml_path = config_dir.join("orchestrator.toml");
        let mut file = std::fs::File::create(&toml_path).unwrap();
        writeln!(
            file,
            r#"
[agent]
auto_assimilate_turn = false
active_toolset = "research"
max_tool_iterations = 5

[gateway.channels.whatsapp]
enabled = true
token_env = "MY_WHATSAPP_TOKEN"

[gateway.channels.matrix]
enabled = false
"#
        )
        .unwrap();

        let cfg = OrchestratorConfig::load_workspace(dir.path()).unwrap();
        assert!(!cfg.agent.auto_assimilate_turn);
        assert_eq!(cfg.agent.active_capability_profile, "research");
        assert_eq!(cfg.agent.max_tool_iterations, 5);

        let whatsapp = cfg
            .gateway
            .extra_channels
            .get("whatsapp")
            .expect("whatsapp");
        assert!(whatsapp.enabled);
        assert_eq!(whatsapp.token_env, "MY_WHATSAPP_TOKEN");

        let matrix = cfg.gateway.extra_channels.get("matrix").expect("matrix");
        assert!(!matrix.enabled);
    }

    #[test]
    fn loads_gateway_from_toml() {
        let dir = tempfile::tempdir().unwrap();
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&config_dir).unwrap();
        let toml_path = config_dir.join("orchestrator.toml");
        let mut file = std::fs::File::create(&toml_path).unwrap();
        writeln!(
            file,
            r#"
[gateway]
bind = "0.0.0.0"
port = 19000
token_env = "MY_GATEWAY_TOKEN"

[gateway.telegram]
enabled = false
"#
        )
        .unwrap();

        let cfg = OrchestratorConfig::load_workspace(dir.path()).unwrap();
        assert_eq!(cfg.gateway.bind, "0.0.0.0");
        assert_eq!(cfg.gateway.port, 19_000);
        assert_eq!(cfg.gateway.token_env, "MY_GATEWAY_TOKEN");
        assert!(!cfg.gateway.telegram.enabled);
    }

    #[test]
    fn skills_hub_defaults_enabled() {
        let cfg = OrchestratorConfig::default();
        assert!(cfg.skills_hub.enabled);
        assert!(cfg.skills_hub.auto_load);
        assert_eq!(cfg.skills_hub.directory, "skills");
    }

    #[test]
    fn loads_skills_hub_from_toml() {
        let dir = tempfile::tempdir().unwrap();
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&config_dir).unwrap();
        let toml_path = config_dir.join("orchestrator.toml");
        let mut file = std::fs::File::create(&toml_path).unwrap();
        writeln!(
            file,
            r#"
[skills_hub]
enabled = true
directory = "custom-skills"
auto_load = false

[[skills_hub.entries]]
id = "inline-echo"
description = "Echo inline"
command = "echo"
args = ["inline"]
enabled = true
stdin_json = false
timeout_secs = 10
"#
        )
        .unwrap();

        let cfg = OrchestratorConfig::load_workspace(dir.path()).unwrap();
        assert_eq!(cfg.skills_hub.directory, "custom-skills");
        assert!(!cfg.skills_hub.auto_load);
        assert_eq!(cfg.skills_hub.entries.len(), 1);
        assert_eq!(cfg.skills_hub.entries[0].id, "inline-echo");
        assert_eq!(cfg.skills_hub_dir(), dir.path().join("custom-skills"));
    }

    #[test]
    fn missing_toml_uses_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = OrchestratorConfig::load_workspace(dir.path()).unwrap();
        assert_eq!(cfg.similarity_thresholds.max_links, 10);
        assert_eq!(cfg.vector_store.store_type, "lancedb");
    }
}
