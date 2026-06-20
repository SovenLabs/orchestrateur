use cortex::{Memory, MemoryId};

use crate::deps::AppDependencies;
use crate::error::OrchestratorError;

/// Use case : récupère une mémoire par identifiant.
pub struct GetMemory {
    deps: AppDependencies,
}

impl GetMemory {
    /// Crée le use case avec les dépendances injectées.
    pub fn new(deps: AppDependencies) -> Self {
        Self { deps }
    }

    /// Charge une mémoire ou retourne `CortexError::MemoryNotFound`.
    pub async fn execute(&self, id: MemoryId) -> Result<Memory, OrchestratorError> {
        Ok(self.deps.memory_repo.get_by_id(id).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockBundle;
    use cortex::{CortexError, Memory, MemoryRepository};

    #[tokio::test]
    async fn gets_existing_memory() {
        let bundle = MockBundle::new();
        let mem = Memory::new("T", "C").unwrap();
        let id = mem.id;
        bundle.memory_repo.save(&mem).await.unwrap();
        let uc = GetMemory::new(bundle.into_deps());
        let loaded = uc.execute(id).await.unwrap();
        assert_eq!(loaded.id, id);
    }

    #[tokio::test]
    async fn missing_memory_returns_error() {
        let bundle = MockBundle::new();
        let uc = GetMemory::new(bundle.into_deps());
        let id = MemoryId::new();
        let err = uc.execute(id).await.unwrap_err();
        assert!(matches!(
            err,
            OrchestratorError::Cortex(CortexError::MemoryNotFound(_))
        ));
    }
}