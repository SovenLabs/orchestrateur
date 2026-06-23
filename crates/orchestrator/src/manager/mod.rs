//! Gestion centralisée des agents persistants.

use std::sync::RwLock;

use crate::communication::{receive_messages, send_message, AgentMessage};
use crate::deps::AppDependencies;
use crate::heartbeat::{run_background_tasks, sleep_agent, wake_agent, BackgroundTaskReport};
use crate::persistent::{
    AgentStructure, CortexAgentBridge, PersistentAgent, PersistentAgentError,
};
use crate::registry::{register_agent, AgentRegistry};

/// API de haut niveau pour créer, lister et piloter les agents persistants.
pub struct AgentManager {
    deps: AppDependencies,
    registry: RwLock<AgentRegistry>,
}

impl AgentManager {
    /// Crée le manager et charge les agents existants depuis le disque.
    ///
    /// # Errors
    ///
    /// Propage [`PersistentAgentError`] si le chargement initial échoue.
    pub async fn new(deps: AppDependencies) -> Result<Self, PersistentAgentError> {
        let mut registry = AgentRegistry::new();
        let agents_dir = deps.config.agents_dir();
        tokio::fs::create_dir_all(&agents_dir).await.map_err(|e| {
            PersistentAgentError::Io(e.to_string())
        })?;
        registry.load_from_disk(&agents_dir).await?;
        registry
            .write_human_registry(&deps.config.agents_registry_path())
            .await?;
        Ok(Self {
            deps,
            registry: RwLock::new(registry),
        })
    }

    /// Construit un manager sans charger le disque (tests).
    #[must_use]
    pub fn for_tests(deps: AppDependencies) -> Self {
        Self {
            deps,
            registry: RwLock::new(AgentRegistry::new()),
        }
    }

    /// Accès aux dépendances injectées.
    #[must_use]
    pub fn deps(&self) -> &AppDependencies {
        &self.deps
    }

    /// Liste tous les agents persistants.
    pub fn list(&self) -> Result<Vec<PersistentAgent>, PersistentAgentError> {
        let guard = self.registry.read().map_err(lock_err)?;
        Ok(guard.list().into_iter().cloned().collect())
    }

    /// Récupère un agent par identifiant.
    pub fn get(&self, id: &str) -> Result<PersistentAgent, PersistentAgentError> {
        let guard = self.registry.read().map_err(lock_err)?;
        guard
            .get(id)
            .cloned()
            .ok_or_else(|| PersistentAgentError::NotFound(id.to_string()))
    }

    /// Crée et enregistre un nouvel agent persistant.
    pub async fn create_agent(
        &self,
        id: &str,
        name: &str,
        role: &str,
        model: Option<&str>,
    ) -> Result<PersistentAgent, PersistentAgentError> {
        let model = model.unwrap_or(&self.deps.config.xai.model);
        let config = AgentStructure::new_config(id, name, role, model);
        let agents_dir = self.deps.config.agents_dir();
        let agent = register_agent(&agents_dir, config).await?;
        CortexAgentBridge::ensure_ready(&self.deps, &agent).await?;

        let mut guard = self.registry.write().map_err(lock_err)?;
        guard.insert(agent.clone());
        guard
            .write_human_registry(&self.deps.config.agents_registry_path())
            .await?;
        Ok(agent)
    }

    /// Réveille un agent (statut `Awake`).
    pub async fn wake(&self, id: &str) -> Result<PersistentAgent, PersistentAgentError> {
        {
            let mut guard = self.registry.write().map_err(lock_err)?;
            let agent = guard
                .get_mut(id)
                .ok_or_else(|| PersistentAgentError::NotFound(id.to_string()))?;
            wake_agent(&self.deps, agent).await?;
        }
        self.sync_registry_entry(id).await?;
        self.get(id)
    }

    /// Met un agent en veille (statut `Sleeping`).
    pub async fn sleep(&self, id: &str) -> Result<PersistentAgent, PersistentAgentError> {
        {
            let mut guard = self.registry.write().map_err(lock_err)?;
            let agent = guard
                .get_mut(id)
                .ok_or_else(|| PersistentAgentError::NotFound(id.to_string()))?;
            sleep_agent(agent).await?;
        }
        self.sync_registry_entry(id).await?;
        self.get(id)
    }

    /// Exécute les tâches de fond d'un agent.
    pub async fn background(&self, id: &str) -> Result<BackgroundTaskReport, PersistentAgentError> {
        let report = {
            let mut guard = self.registry.write().map_err(lock_err)?;
            let agent = guard
                .get_mut(id)
                .ok_or_else(|| PersistentAgentError::NotFound(id.to_string()))?;
            run_background_tasks(&self.deps, agent).await?
        };
        self.sync_registry_entry(id).await?;
        Ok(report)
    }

    /// Envoie un message d'un agent à un autre.
    pub async fn send_message(
        &self,
        from: &str,
        to: &str,
        body: &str,
    ) -> Result<AgentMessage, PersistentAgentError> {
        let _ = self.get(from)?;
        let _ = self.get(to)?;
        send_message(&self.deps.config.agents_dir(), from, to, body).await
    }

    /// Lit l'inbox d'un agent.
    pub async fn receive_messages(
        &self,
        id: &str,
        mark_read: bool,
    ) -> Result<Vec<AgentMessage>, PersistentAgentError> {
        let agent = self.get(id)?;
        receive_messages(&agent.root, mark_read).await
    }

    /// Assimile du texte dans les mémoires propres à l'agent.
    pub async fn assimilate_memory(
        &self,
        id: &str,
        text: &str,
        tags: &[String],
    ) -> Result<cortex::Memory, PersistentAgentError> {
        let agent = self.get(id)?;
        let store = CortexAgentBridge::memory_store(agent, self.deps.clone());
        store.assimilate(text, tags).await
    }
}

impl AgentManager {
    async fn sync_registry_entry(&self, id: &str) -> Result<(), PersistentAgentError> {
        let guard = self.registry.read().map_err(lock_err)?;
        guard.persist_agent(id).await?;
        guard
            .write_human_registry(&self.deps.config.agents_registry_path())
            .await
    }
}

fn lock_err<T>(_: std::sync::PoisonError<T>) -> PersistentAgentError {
    PersistentAgentError::Io("verrou registry empoisonné".into())
}