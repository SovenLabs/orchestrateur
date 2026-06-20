use cortex::Memory;

use crate::deps::AppDependencies;
use crate::error::OrchestratorError;

/// Use case : persiste une mémoire et indexe son embedding.
pub struct SaveMemory {
    deps: AppDependencies,
}

impl SaveMemory {
    /// Crée le use case avec les dépendances injectées.
    #[must_use]
    pub fn new(deps: AppDependencies) -> Self {
        Self { deps }
    }

    /// Sauvegarde la mémoire et met à jour le vector store.
    ///
    /// # Errors
    ///
    /// Propage une [`OrchestratorError`] si la persistance ou l'indexation échoue.
    pub async fn execute(&self, memory: &Memory) -> Result<Memory, OrchestratorError> {
        tracing::debug!(memory_id = %memory.id, title = %memory.title, "save_memory");
        let embedding = self
            .deps
            .embedding
            .embed(&format!("{} {}", memory.title, memory.content))
            .await?;
        self.deps.memory_repo.save(memory).await?;
        self.deps
            .vector_store
            .upsert(memory.id, &embedding)
            .await?;
        Ok(memory.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::testing::MockBundle;
    use cortex::{Memory, MemoryRepository};

    #[tokio::test]
    async fn save_persists_and_indexes() {
        let bundle = MockBundle::new();
        let mem = Memory::new("T", "C").unwrap();
        let id = mem.id;
        let repo = Arc::clone(&bundle.memory_repo);
        let store = Arc::clone(&bundle.vector_store);
        let uc = SaveMemory::new(bundle.into_deps());
        uc.execute(&mem).await.unwrap();
        assert_eq!(repo.len(), 1);
        assert_eq!(store.len(), 1);
        let loaded = repo.get_by_id(id).await.unwrap();
        assert_eq!(loaded.title, "T");
    }
}