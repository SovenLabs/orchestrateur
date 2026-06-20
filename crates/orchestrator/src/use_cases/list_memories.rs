use cortex::Memory;

use crate::deps::AppDependencies;
use crate::error::OrchestratorError;

/// Use case : liste toutes les mémoires persistées.
pub struct ListMemories {
    deps: AppDependencies,
}

impl ListMemories {
    /// Crée le use case avec les dépendances injectées.
    pub fn new(deps: AppDependencies) -> Self {
        Self { deps }
    }

    /// Retourne toutes les mémoires (ordre non garanti).
    pub async fn execute(&self) -> Result<Vec<Memory>, OrchestratorError> {
        Ok(self.deps.memory_repo.list().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockBundle;
    use cortex::{Memory, MemoryRepository};

    #[tokio::test]
    async fn lists_saved_memories() {
        let bundle = MockBundle::new();
        let mem = Memory::new("A", "contenu a").unwrap();
        bundle.memory_repo.save(&mem).await.unwrap();
        let uc = ListMemories::new(bundle.into_deps());
        let list = uc.execute().await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title, "A");
    }

    #[tokio::test]
    async fn empty_repo_returns_empty_list() {
        let bundle = MockBundle::new();
        let uc = ListMemories::new(bundle.into_deps());
        assert!(uc.execute().await.unwrap().is_empty());
    }
}