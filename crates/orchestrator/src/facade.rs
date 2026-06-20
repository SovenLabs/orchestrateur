use cortex::{Memory, MemoryId, SearchFilter, SearchHit};

use crate::deps::AppDependencies;
use crate::error::{OrchestratorError, SkillError};
use crate::memory_draft::MemoryDraft;
use crate::skills::{SkillContext, SkillOutput, SkillRegistry};
use crate::use_cases::{
    AssimilateFromDraft, AssimilationResult, GetMemory, ListMemories, SaveMemory, SearchMemories,
};

/// Point d'entrée public stable de l'orchestrateur (CLI, GUI, tests d'intégration).
///
/// Ne contient aucune logique métier : délègue aux use cases.
pub struct OrchestratorFacade {
    deps: AppDependencies,
    skills: SkillRegistry,
}

impl OrchestratorFacade {
    /// Construit la facade avec dépendances injectées et registre de skills par défaut.
    pub fn new(deps: AppDependencies) -> Self {
        Self {
            deps,
            skills: SkillRegistry::with_defaults(),
        }
    }

    /// Construit la facade avec un registre de skills personnalisé.
    pub fn with_skills(deps: AppDependencies, skills: SkillRegistry) -> Self {
        Self { deps, skills }
    }

    /// Accès en lecture aux dépendances (tests / composition).
    pub fn deps(&self) -> &AppDependencies {
        &self.deps
    }

    /// Accès au registre de skills.
    pub fn skills(&self) -> &SkillRegistry {
        &self.skills
    }

    /// Liste toutes les mémoires.
    pub async fn list_memories(&self) -> Result<Vec<Memory>, OrchestratorError> {
        ListMemories::new(self.deps.clone()).execute().await
    }

    /// Récupère une mémoire par identifiant.
    pub async fn get_memory(&self, id: MemoryId) -> Result<Memory, OrchestratorError> {
        GetMemory::new(self.deps.clone()).execute(id).await
    }

    /// Persiste une mémoire et indexe son embedding.
    pub async fn save_memory(&self, memory: &Memory) -> Result<Memory, OrchestratorError> {
        SaveMemory::new(self.deps.clone()).execute(memory).await
    }

    /// Recherche hybride par requête textuelle.
    pub async fn search_memories(
        &self,
        query: &str,
        filter: &SearchFilter,
    ) -> Result<Vec<SearchHit>, OrchestratorError> {
        SearchMemories::new(self.deps.clone()).execute(query, filter).await
    }

    /// Assimile un brouillon (dry-run Phase 2, sans appel IA).
    pub async fn assimilate_from_draft(
        &self,
        draft: MemoryDraft,
    ) -> Result<AssimilationResult, OrchestratorError> {
        AssimilateFromDraft::new(self.deps.clone())
            .execute(draft)
            .await
    }

    /// Liste les skills enregistrées (nom, description).
    pub fn list_skills(&self) -> Vec<(&'static str, &'static str)> {
        self.skills.list()
    }

    /// Exécute une skill par son nom.
    pub async fn execute_skill(
        &self,
        name: &str,
        ctx: &SkillContext,
    ) -> Result<SkillOutput, SkillError> {
        self.skills.execute(name, ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_draft::MemoryDraft;
    use crate::testing::MockBundle;
    use cortex::{DomainEvent, Memory, SearchFilter};

    fn facade() -> OrchestratorFacade {
        OrchestratorFacade::new(MockBundle::new().into_deps())
    }

    #[tokio::test]
    async fn facade_lists_and_gets_memory() {
        let f = facade();
        let mem = Memory::new("T", "C").unwrap();
        let id = mem.id;
        f.save_memory(&mem).await.unwrap();
        assert_eq!(f.list_memories().await.unwrap().len(), 1);
        assert_eq!(f.get_memory(id).await.unwrap().id, id);
    }

    #[tokio::test]
    async fn facade_searches_memories() {
        let f = facade();
        let mem = Memory::new("Recherche", "Contenu indexé.").unwrap();
        f.save_memory(&mem).await.unwrap();
        let hits = f
            .search_memories("indexé", &SearchFilter::default())
            .await
            .unwrap();
        assert!(!hits.is_empty());
    }

    #[tokio::test]
    async fn facade_assimilates_draft() {
        let f = facade();
        let draft = MemoryDraft {
            title: "Nouveau".into(),
            content: "Souvenir assimilé.".into(),
            tags: vec![],
            backlinks: vec![],
        };
        let (memory, events) = f.assimilate_from_draft(draft).await.unwrap();
        assert_eq!(memory.title, "Nouveau");
        assert!(matches!(events[0], DomainEvent::MemoryAssimilated(_)));
    }

    #[tokio::test]
    async fn facade_lists_and_executes_noop_skill() {
        let f = facade();
        let skills = f.list_skills();
        assert!(skills.iter().any(|(n, _)| *n == "noop"));
        let out = f.execute_skill("noop", &SkillContext).await.unwrap();
        assert_eq!(out.message, "noop ok");
    }
}