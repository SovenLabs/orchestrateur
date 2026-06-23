//! Mémoires propres à un agent persistant (`agents/{id}/memories/`).

use std::path::Path;
use std::sync::Arc;

use cortex::{
    parse_memory_markdown, serialize_memory, Memory, MemoryRepository, SearchFilter,
};

use crate::deps::AppDependencies;
use crate::use_cases::{AssimilateFromText, ListMemories, SearchMemories};

use super::PersistentAgent;
use super::PersistentAgentError;

/// Repository fichier léger pour le dossier mémoires d'un agent.
struct AgentFileMemoryRepository {
    dir: std::path::PathBuf,
}

#[async_trait::async_trait]
impl MemoryRepository for AgentFileMemoryRepository {
    async fn save(&self, memory: &Memory) -> Result<(), cortex::CortexError> {
        tokio::fs::create_dir_all(&self.dir).await.map_err(|e| {
            cortex::CortexError::GraphError(format!("création {}: {e}", self.dir.display()))
        })?;
        let markdown = serialize_memory(memory)?;
        let path = self.dir.join(format!("{}.md", memory.id));
        tokio::fs::write(&path, markdown).await.map_err(|e| {
            cortex::CortexError::GraphError(format!("écriture {}: {e}", path.display()))
        })?;
        Ok(())
    }

    async fn get_by_id(&self, id: cortex::MemoryId) -> Result<Memory, cortex::CortexError> {
        let path = self.dir.join(format!("{id}.md"));
        let raw = tokio::fs::read_to_string(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                cortex::CortexError::MemoryNotFound(id)
            } else {
                cortex::CortexError::GraphError(format!("lecture {}: {e}", path.display()))
            }
        })?;
        Ok(parse_memory_markdown(&raw)?.memory)
    }

    async fn list(&self) -> Result<Vec<Memory>, cortex::CortexError> {
        if !self.dir.exists() {
            return Ok(Vec::new());
        }
        let mut entries = tokio::fs::read_dir(&self.dir).await.map_err(|e| {
            cortex::CortexError::GraphError(format!("listage {}: {e}", self.dir.display()))
        })?;
        let mut memories = Vec::new();
        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            cortex::CortexError::GraphError(e.to_string())
        })? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let raw = tokio::fs::read_to_string(&path).await.map_err(|e| {
                cortex::CortexError::GraphError(format!("lecture {}: {e}", path.display()))
            })?;
            memories.push(parse_memory_markdown(&raw)?.memory);
        }
        Ok(memories)
    }

    async fn delete(&self, id: cortex::MemoryId) -> Result<(), cortex::CortexError> {
        let path = self.dir.join(format!("{id}.md"));
        tokio::fs::remove_file(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                cortex::CortexError::MemoryNotFound(id)
            } else {
                cortex::CortexError::GraphError(format!("suppression {}: {e}", path.display()))
            }
        })?;
        Ok(())
    }
}

/// Accès Cortex scopingé au dossier mémoires d'un agent.
pub struct AgentMemoryStore {
    agent: PersistentAgent,
    deps: AppDependencies,
}

impl AgentMemoryStore {
    /// Crée un store pour l'agent donné.
    #[must_use]
    pub fn new(agent: PersistentAgent, deps: AppDependencies) -> Self {
        Self { agent, deps }
    }

    /// Répertoire mémoires de l'agent.
    #[must_use]
    pub fn memories_dir(&self) -> std::path::PathBuf {
        self.agent.root.join("memories")
    }

    /// Repository fichier dédié à cet agent.
    #[must_use]
    pub fn repository(&self) -> Arc<dyn MemoryRepository> {
        Arc::new(AgentFileMemoryRepository {
            dir: self.memories_dir(),
        })
    }

    fn scoped_deps(&self) -> AppDependencies {
        let mut scoped = self.deps.clone();
        scoped.memory_repo = self.repository();
        scoped
    }

    /// Assimile du texte dans les mémoires de l'agent.
    pub async fn assimilate(&self, text: &str, tags: &[String]) -> Result<Memory, PersistentAgentError> {
        let (memory, _) = AssimilateFromText::new(self.scoped_deps())
            .execute(text, tags, None)
            .await?;
        Ok(memory)
    }

    /// Liste les mémoires de l'agent.
    pub async fn list(&self) -> Result<Vec<Memory>, PersistentAgentError> {
        Ok(ListMemories::new(self.scoped_deps()).execute().await?)
    }

    /// Recherche sémantique dans les mémoires de l'agent (vector store global).
    pub async fn search(
        &self,
        query: &str,
        filter: &SearchFilter,
    ) -> Result<Vec<cortex::SearchHit>, PersistentAgentError> {
        Ok(SearchMemories::new(self.scoped_deps())
            .execute(query, filter)
            .await?)
    }

    /// Compte les fichiers `.md` dans le dossier mémoires.
    pub async fn count_files(&self) -> Result<usize, PersistentAgentError> {
        Ok(self.repository().list().await?.len())
    }
}

/// Vérifie que le répertoire mémoires existe.
pub async fn ensure_memories_dir(agent_root: &Path) -> Result<(), PersistentAgentError> {
    let dir = agent_root.join("memories");
    tokio::fs::create_dir_all(&dir).await.map_err(|e| {
        PersistentAgentError::Io(format!("création {}: {e}", dir.display()))
    })
}