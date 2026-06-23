//! Arborescence standard d'un agent persistant sur disque.

use std::path::{Path, PathBuf};

use chrono::Utc;
use tokio::fs;

use super::identity::AgentStatus;
use super::{PersistentAgent, PersistentAgentConfig, PersistentAgentError};

/// Dossiers et fichiers créés pour chaque agent.
pub struct AgentStructure;

impl AgentStructure {
    /// Répertoire racine d'un agent (`workspace/agents/{id}/`).
    #[must_use]
    pub fn agent_root(agents_dir: &Path, id: &str) -> PathBuf {
        agents_dir.join(id)
    }

    /// Crée l'arborescence complète et les fichiers initiaux.
    ///
    /// # Errors
    ///
    /// Propage [`PersistentAgentError`] si l'écriture disque échoue.
    pub async fn scaffold(
        agents_dir: &Path,
        config: &PersistentAgentConfig,
    ) -> Result<PathBuf, PersistentAgentError> {
        let root = Self::agent_root(agents_dir, &config.id);
        if root.exists() {
            return Err(PersistentAgentError::AlreadyExists(config.id.clone()));
        }

        let dirs = [
            root.clone(),
            root.join("tasks"),
            root.join("memories"),
            root.join("messages"),
            root.join("messages").join("inbox"),
            root.join("messages").join("outbox"),
        ];
        for dir in dirs {
            fs::create_dir_all(&dir).await.map_err(|e| {
                PersistentAgentError::Io(format!("création {}: {e}", dir.display()))
            })?;
        }

        let personality = format!(
            "# Personality\n\n\
             Tu es **{}**, agent **{}** de l'écosystème Orchestrateur.\n\
             Ton rôle : {}.\n",
            config.name, config.id, config.role
        );
        fs::write(root.join("personality.md"), personality)
            .await
            .map_err(io_err)?;

        let heartbeat = format!(
            "# Heartbeat\n\n\
             interval_secs: 300\n\
             last_wake: null\n\
             last_sleep: null\n\
             background_tasks:\n\
               - check_inbox\n\
               - scan_tasks\n"
        );
        fs::write(root.join("heartbeat.md"), heartbeat)
            .await
            .map_err(io_err)?;

        Self::write_config(&root, config).await?;
        Ok(root)
    }

    /// Persiste `config.toml` dans le dossier agent.
    pub async fn write_config(
        root: &Path,
        config: &PersistentAgentConfig,
    ) -> Result<(), PersistentAgentError> {
        let toml = toml::to_string_pretty(config).map_err(|e| PersistentAgentError::Config(e.to_string()))?;
        fs::write(root.join("config.toml"), toml)
            .await
            .map_err(io_err)
    }

    /// Charge `config.toml` depuis le dossier agent.
    pub async fn load_config(root: &Path) -> Result<PersistentAgentConfig, PersistentAgentError> {
        let path = root.join("config.toml");
        let raw = fs::read_to_string(&path).await.map_err(|e| {
            PersistentAgentError::Io(format!("lecture {}: {e}", path.display()))
        })?;
        toml::from_str(&raw).map_err(|e| PersistentAgentError::Config(e.to_string()))
    }

    /// Construit une config initiale pour un nouvel agent.
    #[must_use]
    pub fn new_config(
        id: impl Into<String>,
        name: impl Into<String>,
        role: impl Into<String>,
        model: impl Into<String>,
    ) -> PersistentAgentConfig {
        let id = id.into();
        PersistentAgentConfig {
            id: id.clone(),
            name: name.into(),
            role: role.into(),
            model: model.into(),
            status: AgentStatus::Sleeping,
            session_key: format!("agent-{id}"),
            created_at: Utc::now().to_rfc3339(),
            last_heartbeat: None,
        }
    }

    /// Instancie [`PersistentAgent`] depuis un dossier existant.
    pub async fn load_agent(
        agents_dir: &Path,
        id: &str,
    ) -> Result<PersistentAgent, PersistentAgentError> {
        let root = Self::agent_root(agents_dir, id);
        if !root.join("config.toml").exists() {
            return Err(PersistentAgentError::NotFound(id.to_string()));
        }
        let config = Self::load_config(&root).await?;
        Ok(PersistentAgent::from_config(root, config))
    }
}

fn io_err(e: std::io::Error) -> PersistentAgentError {
    PersistentAgentError::Io(e.to_string())
}