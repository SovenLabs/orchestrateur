use cortex::{Memory, MemoryId, SearchFilter, SearchHit};

use crate::deps::AppDependencies;
use crate::error::{OrchestratorError, SkillError};
use crate::llm::ChatMessage;
use crate::memory_draft::MemoryDraft;
use crate::skills::{SkillContext, SkillOutput, SkillRegistry};
use std::path::Path;

use crate::use_cases::{
    AssimilateFromDraft, AssimilateFromText, AssimilationResult, GetMemory, ImportMemories,
    ImportResult, ListMemories, SaveMemory, SearchMemories,
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
    #[must_use]
    pub fn new(deps: AppDependencies) -> Self {
        let skills = SkillRegistry::with_operational_skills(deps.clone());
        Self { deps, skills }
    }

    /// Construit la facade avec un registre de skills personnalisé.
    #[must_use]
    pub fn with_skills(deps: AppDependencies, skills: SkillRegistry) -> Self {
        Self { deps, skills }
    }

    /// Accès en lecture aux dépendances (tests / composition).
    #[must_use]
    pub fn deps(&self) -> &AppDependencies {
        &self.deps
    }

    /// Accès au registre de skills.
    #[must_use]
    pub fn skills(&self) -> &SkillRegistry {
        &self.skills
    }

    /// Liste toutes les mémoires.
    ///
    /// # Errors
    ///
    /// Propage une [`OrchestratorError`] si le port échoue.
    pub async fn list_memories(&self) -> Result<Vec<Memory>, OrchestratorError> {
        ListMemories::new(self.deps.clone()).execute().await
    }

    /// Récupère une mémoire par identifiant.
    ///
    /// # Errors
    ///
    /// Propage une [`OrchestratorError`] si la mémoire est introuvable ou si le port échoue.
    pub async fn get_memory(&self, id: MemoryId) -> Result<Memory, OrchestratorError> {
        GetMemory::new(self.deps.clone()).execute(id).await
    }

    /// Persiste une mémoire et indexe son embedding.
    ///
    /// # Errors
    ///
    /// Propage une [`OrchestratorError`] si la persistance ou l'indexation échoue.
    pub async fn save_memory(&self, memory: &Memory) -> Result<Memory, OrchestratorError> {
        SaveMemory::new(self.deps.clone()).execute(memory).await
    }

    /// Recherche hybride par requête textuelle.
    ///
    /// # Errors
    ///
    /// Propage une [`OrchestratorError`] si l'embedding ou la recherche échoue.
    pub async fn search_memories(
        &self,
        query: &str,
        filter: &SearchFilter,
    ) -> Result<Vec<SearchHit>, OrchestratorError> {
        SearchMemories::new(self.deps.clone())
            .execute(query, filter)
            .await
    }

    /// Assimile un brouillon pré-construit (sans appel LLM).
    ///
    /// # Errors
    ///
    /// Propage une [`OrchestratorError`] si la validation, le graphe ou la persistance échoue.
    pub async fn assimilate_from_draft(
        &self,
        draft: MemoryDraft,
    ) -> Result<AssimilationResult, OrchestratorError> {
        AssimilateFromDraft::new(self.deps.clone())
            .execute(draft)
            .await
    }

    /// Assimile du texte brut via le provider LLM configuré (flux opérationnel Phase 3).
    ///
    /// # Errors
    ///
    /// Propage une [`OrchestratorError`] si le LLM, la validation ou la persistance échoue.
    pub async fn assimilate(
        &self,
        user_prompt: &str,
        system_prompt: Option<&str>,
    ) -> Result<AssimilationResult, OrchestratorError> {
        AssimilateFromText::new(self.deps.clone())
            .execute(user_prompt, system_prompt)
            .await
    }

    /// Liste les skills enregistrées (nom, description).
    #[must_use]
    pub fn list_skills(&self) -> Vec<(&'static str, &'static str)> {
        self.skills.list()
    }

    /// Chat libre avec le provider LLM configuré.
    ///
    /// # Errors
    ///
    /// Propage [`OrchestratorError::Llm`] si le provider échoue.
    pub async fn chat(&self, message: &str) -> Result<String, OrchestratorError> {
        let reply = self
            .deps
            .llm
            .chat(&[ChatMessage {
                role: "user".into(),
                content: message.into(),
            }])
            .await?;
        if let Some(usage) = self.deps.llm.last_usage() {
            self.deps.events.publish_llm_usage(&usage);
        }
        Ok(reply)
    }

    /// Importe des mémoires Markdown depuis un répertoire (`*.md`).
    ///
    /// # Errors
    ///
    /// Propage une [`OrchestratorError`] si la lecture du répertoire échoue.
    pub async fn import_from_directory(
        &self,
        source_dir: &Path,
    ) -> Result<ImportResult, OrchestratorError> {
        ImportMemories::new(self.deps.clone())
            .execute(source_dir)
            .await
    }

    /// Exécute une skill par son nom.
    ///
    /// # Errors
    ///
    /// Retourne [`SkillError::NotFound`] ou [`SkillError::ExecutionFailed`].
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
        assert!(events
            .iter()
            .any(|e| matches!(e, DomainEvent::MemoryAssimilated(_))));
    }

    #[tokio::test]
    async fn facade_lists_and_executes_noop_skill() {
        let f = facade();
        let skills = f.list_skills();
        assert!(skills.iter().any(|(n, _)| *n == "noop"));
        let out = f
            .execute_skill("noop", &SkillContext::default())
            .await
            .unwrap();
        assert_eq!(out.message, "noop ok");
    }

    #[tokio::test]
    async fn facade_executes_operational_skills() {
        let f = facade();
        let skills = f.list_skills();
        assert!(skills.iter().any(|(n, _)| *n == "list_memories"));
        assert!(skills.iter().any(|(n, _)| *n == "search"));
        assert!(skills.iter().any(|(n, _)| *n == "assimilate"));

        let mem = Memory::new("Skill", "contenu skill").unwrap();
        f.save_memory(&mem).await.unwrap();

        let out = f
            .execute_skill("list_memories", &SkillContext::default())
            .await
            .unwrap();
        assert!(out.message.contains("Skill"));

        let out = f
            .execute_skill(
                "search",
                &SkillContext {
                    query: Some("contenu".into()),
                    ..SkillContext::default()
                },
            )
            .await
            .unwrap();
        assert!(out.message.contains("résultat"));

        let out = f
            .execute_skill(
                "assimilate",
                &SkillContext {
                    text: Some("Texte à assimiler via facade.".into()),
                    ..SkillContext::default()
                },
            )
            .await
            .unwrap();
        assert!(out.message.contains("Assimilé"));
    }
}
