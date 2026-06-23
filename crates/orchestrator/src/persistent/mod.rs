//! Agents persistants — identité, dossier, cycle de vie indépendant des sessions.

mod cortex_bridge;
mod identity;
mod memory;
mod structure;

pub use cortex_bridge::CortexAgentBridge;
pub use identity::{AgentIdentity, AgentStatus};
pub use memory::AgentMemoryStore;
pub use structure::AgentStructure;

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use cortex::SessionKey;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Configuration sérialisée dans `config.toml` de chaque agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistentAgentConfig {
    /// Identifiant stable (nom de dossier).
    pub id: String,
    /// Nom affiché.
    pub name: String,
    /// Rôle fonctionnel.
    pub role: String,
    /// Modèle LLM associé.
    pub model: String,
    /// Statut du cycle de vie.
    pub status: AgentStatus,
    /// Clé de session SQLite liée à cet agent.
    pub session_key: String,
    /// Horodatage de création ISO-8601.
    pub created_at: String,
    /// Dernier heartbeat enregistré.
    pub last_heartbeat: Option<String>,
}

/// Entité principale d'un agent persistant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistentAgent {
    /// Racine du dossier agent.
    pub root: PathBuf,
    /// Configuration persistée.
    pub config: PersistentAgentConfig,
}

impl PersistentAgent {
    /// Construit l'entité depuis config + chemin racine.
    #[must_use]
    pub fn from_config(root: PathBuf, config: PersistentAgentConfig) -> Self {
        Self { root, config }
    }

    /// Met à jour le statut en mémoire (persistance via [`AgentStructure::write_config`]).
    pub fn set_status(&mut self, status: AgentStatus) {
        self.config.status = status;
    }

    /// Met à jour le timestamp de heartbeat.
    pub fn touch_heartbeat(&mut self) {
        self.config.last_heartbeat = Some(Utc::now().to_rfc3339());
    }

    /// Parse le dernier heartbeat si présent.
    #[must_use]
    pub fn last_heartbeat_at(&self) -> Option<DateTime<Utc>> {
        self.config
            .last_heartbeat
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
    }

    /// Clé de session Cortex associée.
    ///
    /// # Errors
    ///
    /// Propage [`PersistentAgentError::InvalidSessionKey`] si la clé est invalide.
    pub fn session_key(&self) -> Result<SessionKey, PersistentAgentError> {
        SessionKey::new(&self.config.session_key).map_err(|e| {
            PersistentAgentError::InvalidSessionKey(e.to_string())
        })
    }
}

impl AgentIdentity for PersistentAgent {
    fn id(&self) -> &str {
        &self.config.id
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn role(&self) -> &str {
        &self.config.role
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    fn status(&self) -> AgentStatus {
        self.config.status
    }
}

/// Erreurs du sous-système agents persistants.
#[derive(Debug, Error)]
pub enum PersistentAgentError {
    /// Agent introuvable.
    #[error("agent introuvable: {0}")]
    NotFound(String),
    /// Agent déjà enregistré.
    #[error("agent déjà enregistré: {0}")]
    AlreadyExists(String),
    /// Erreur de configuration TOML.
    #[error("configuration agent: {0}")]
    Config(String),
    /// Erreur I/O disque.
    #[error("IO: {0}")]
    Io(String),
    /// Clé de session invalide.
    #[error("clé de session invalide: {0}")]
    InvalidSessionKey(String),
    /// Erreur Cortex.
    #[error("cortex: {0}")]
    Cortex(#[from] cortex::CortexError),
    /// Erreur orchestrateur.
    #[error("orchestrateur: {0}")]
    Orchestrator(#[from] crate::error::OrchestratorError),
}