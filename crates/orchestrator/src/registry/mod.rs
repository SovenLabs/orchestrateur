//! Registre central des agents persistants.

mod loader;
mod register;

pub use loader::load_agents;
pub use register::register_agent;

use std::collections::HashMap;
use std::path::Path;

use tokio::fs;

use crate::persistent::{AgentIdentity, AgentStructure, PersistentAgent, PersistentAgentError};

/// Liste centrale des agents persistants (mémoire + synchronisation disque).
#[derive(Debug, Default)]
pub struct AgentRegistry {
    agents: HashMap<String, PersistentAgent>,
}

impl AgentRegistry {
    /// Crée un registre vide.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Charge les agents depuis le disque dans le registre.
    pub async fn load_from_disk(
        &mut self,
        agents_dir: &Path,
    ) -> Result<usize, PersistentAgentError> {
        let loaded = load_agents(agents_dir).await?;
        let count = loaded.len();
        for agent in loaded {
            self.agents.insert(agent.id().to_string(), agent);
        }
        Ok(count)
    }

    /// Enregistre un agent (mémoire + disque déjà créé par l'appelant).
    pub fn insert(&mut self, agent: PersistentAgent) {
        self.agents.insert(agent.id().to_string(), agent);
    }

    /// Retourne une référence immuable par identifiant.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&PersistentAgent> {
        self.agents.get(id)
    }

    /// Retourne une référence mutable par identifiant.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut PersistentAgent> {
        self.agents.get_mut(id)
    }

    /// Liste triée par identifiant.
    #[must_use]
    pub fn list(&self) -> Vec<&PersistentAgent> {
        let mut items: Vec<_> = self.agents.values().collect();
        items.sort_by_key(|a| a.id());
        items
    }

    /// Nombre d'agents enregistrés.
    #[must_use]
    pub fn len(&self) -> usize {
        self.agents.len()
    }

    /// Indique si le registre est vide.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }

    /// Persiste la config d'un agent sur disque.
    pub async fn persist_agent(&self, id: &str) -> Result<(), PersistentAgentError> {
        let agent = self
            .agents
            .get(id)
            .ok_or_else(|| PersistentAgentError::NotFound(id.to_string()))?;
        AgentStructure::write_config(&agent.root, &agent.config).await
    }

    /// Régénère le fichier `AGENTS_REGISTRY.md` (vue humaine).
    pub async fn write_human_registry(
        &self,
        registry_path: &Path,
    ) -> Result<(), PersistentAgentError> {
        if let Some(parent) = registry_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                PersistentAgentError::Io(format!("création {}: {e}", parent.display()))
            })?;
        }

        let mut lines = vec![
            "# Registre des agents persistants".to_string(),
            String::new(),
            "| ID | Nom | Rôle | Modèle | Statut | Dernier heartbeat |".to_string(),
            "|----|-----|------|--------|--------|-------------------|".to_string(),
        ];

        for agent in self.list() {
            let hb = agent.config.last_heartbeat.as_deref().unwrap_or("—");
            lines.push(format!(
                "| {} | {} | {} | {} | {} | {} |",
                agent.id(),
                agent.name(),
                agent.role(),
                agent.model(),
                agent.status().label(),
                hb,
            ));
        }

        if self.is_empty() {
            lines.push(String::new());
            lines.push("_Aucun agent enregistré._".to_string());
        }

        fs::write(registry_path, lines.join("\n"))
            .await
            .map_err(|e| PersistentAgentError::Io(e.to_string()))
    }
}