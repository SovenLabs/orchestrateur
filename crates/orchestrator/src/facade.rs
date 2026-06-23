use cortex::{Memory, MemoryId, SearchFilter, SearchHit, SessionKey};
use crate::agent::{
    AgentConfig, AgentError, AgentLoop, AgentStreamSink, AgentTurnRequest, AgentTurnResult,
};

use crate::deps::AppDependencies;
use crate::error::{OrchestratorError, SkillError};
use crate::llm::ChatMessage;
use crate::memory_draft::MemoryDraft;
use crate::skills::{SkillContext, SkillEntry, SkillOutput, SkillRegistry};
use std::path::Path;
use std::sync::Arc;

use crate::b212::{
    ensure_b212_agents, relay_workflow_steps, wake_b212_agents_for_workflow, B212AnalyzeRequest,
    B212GovernanceService, B212WorkflowResult, B212WorkflowService,
};
use crate::bridge::DraftSummary;
use crate::draft::{DraftError, DraftStatus, StoredDraft};
use crate::manager::AgentManager;
use crate::persistent::PersistentAgentError;
use crate::use_cases::{
    AssimilateFromDraft, AssimilateFromText, AssimilationResult, GetMemory, ImportMemories,
    ImportResult, ListMemories, SaveMemory, SearchMemories,
};

/// Point d'entrée public stable de l'orchestrateur (CLI, GUI, tests d'intégration).
///
/// Ne contient aucune logique métier : délègue aux use cases.
pub struct OrchestratorFacade {
    deps: AppDependencies,
    skills: Arc<SkillRegistry>,
}

impl OrchestratorFacade {
    /// Construit la facade avec dépendances injectées et registre de skills par défaut.
    #[must_use]
    pub fn new(deps: AppDependencies) -> Self {
        let skills = Arc::new(SkillRegistry::with_operational_skills_and_hub(deps.clone()));
        Self { deps, skills }
    }

    /// Construit la facade avec un registre de skills personnalisé.
    #[must_use]
    pub fn with_skills(deps: AppDependencies, skills: SkillRegistry) -> Self {
        Self {
            deps,
            skills: Arc::new(skills),
        }
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

    /// Accès partagé au registre (outils agent / gateway).
    #[must_use]
    pub fn skills_registry(&self) -> Arc<SkillRegistry> {
        Arc::clone(&self.skills)
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
        text: &str,
        tags: &[String],
        system_prompt: Option<&str>,
    ) -> Result<AssimilationResult, OrchestratorError> {
        AssimilateFromText::new(self.deps.clone())
            .execute(text, tags, system_prompt)
            .await
    }

    /// Liste les brouillons `pending` en attente de publication.
    ///
    /// # Errors
    ///
    /// Propage [`OrchestratorError`] si la lecture échoue.
    pub async fn list_drafts(&self) -> Result<Vec<DraftSummary>, OrchestratorError> {
        let drafts = self
            .deps
            .draft_repo
            .list(Some(DraftStatus::Pending))
            .await?;
        Ok(drafts.iter().map(StoredDraft::to_summary).collect())
    }

    /// Récupère un brouillon par identifiant.
    ///
    /// # Errors
    ///
    /// Propage [`OrchestratorError`] si le brouillon est introuvable.
    pub async fn get_draft(&self, id: &str) -> Result<StoredDraft, OrchestratorError> {
        self.deps.draft_repo.get_by_id(id).await.map_err(Into::into)
    }

    /// Persiste un nouveau brouillon `pending` (watcher / pipeline insight).
    ///
    /// # Errors
    ///
    /// Propage [`OrchestratorError`] si la persistance échoue.
    pub async fn store_draft(
        &self,
        draft: MemoryDraft,
        watcher_session: Option<String>,
    ) -> Result<StoredDraft, OrchestratorError> {
        self.deps
            .draft_repo
            .create_pending(draft, watcher_session)
            .await
            .map_err(Into::into)
    }

    /// Publie un brouillon (assimilation Cortex) et marque le statut `published`.
    ///
    /// # Errors
    ///
    /// Propage [`OrchestratorError`] si le brouillon est introuvable ou l'assimilation échoue.
    pub async fn publish_draft(
        &self,
        id: &str,
    ) -> Result<(String, AssimilationResult), OrchestratorError> {
        let stored = self.deps.draft_repo.get_by_id(id).await?;
        if stored.status != DraftStatus::Pending {
            return Err(DraftError::InvalidStatus {
                expected: DraftStatus::Pending.as_str(),
                actual: stored.status.as_str(),
            }
            .into());
        }
        let result = AssimilateFromDraft::new(self.deps.clone())
            .execute(stored.draft)
            .await?;
        self.deps
            .draft_repo
            .update_status(id, DraftStatus::Published)
            .await?;
        Ok((id.to_string(), result))
    }

    /// Marque un brouillon `discarded` sans publier.
    ///
    /// # Errors
    ///
    /// Propage [`OrchestratorError`] si le brouillon est introuvable.
    pub async fn discard_draft(&self, id: &str) -> Result<(), OrchestratorError> {
        let stored = self.deps.draft_repo.get_by_id(id).await?;
        if stored.status != DraftStatus::Pending {
            return Err(DraftError::InvalidStatus {
                expected: DraftStatus::Pending.as_str(),
                actual: stored.status.as_str(),
            }
            .into());
        }
        self.deps
            .draft_repo
            .update_status(id, DraftStatus::Discarded)
            .await?;
        Ok(())
    }

    /// Liste les skills enregistrées (métadonnées complètes).
    #[must_use]
    pub fn list_skills(&self) -> Vec<SkillEntry> {
        self.skills.list()
    }

    /// Construit le gestionnaire d'agents persistants (Phase 2).
    ///
    /// # Errors
    ///
    /// Propage [`PersistentAgentError`] si le chargement initial échoue.
    pub async fn agent_manager(&self) -> Result<AgentManager, PersistentAgentError> {
        AgentManager::new(self.deps.clone()).await
    }

    /// Service workflow B212 (Phase 3).
    ///
    /// # Errors
    ///
    /// Propage [`b212::B212Error`] si B212 est désactivé ou non câblé.
    pub fn b212_workflow_service(&self) -> Result<B212WorkflowService, b212::B212Error> {
        B212WorkflowService::new(self.deps.clone())
    }

    /// Initialise les 6 agents domaine B212 (idempotent).
    pub async fn b212_init_agents(&self) -> Result<Vec<String>, PersistentAgentError> {
        let manager = self.agent_manager().await?;
        let agents = ensure_b212_agents(&manager).await?;
        Ok(agents.iter().map(|a| a.config.id.clone()).collect())
    }

    /// Exécute le workflow B212 complet avec agents persistants réveillés.
    pub async fn b212_analyze(
        &self,
        req: B212AnalyzeRequest,
    ) -> Result<B212WorkflowResult, b212::B212Error> {
        let service = self.b212_workflow_service()?;
        let manager = self
            .agent_manager()
            .await
            .map_err(|e| b212::B212Error::Config(e.to_string()))?;
        wake_b212_agents_for_workflow(&manager)
            .await
            .map_err(|e| b212::B212Error::Config(e.to_string()))?;
        let result = service.run(req).await?;
        let _ = relay_workflow_steps(&manager, &result.steps)
            .await
            .map_err(|e| b212::B212Error::Journal(e.to_string()));
        Ok(result)
    }

    /// Liste les propositions trade en attente HITL.
    pub async fn b212_list_pending_proposals(
        &self,
    ) -> Result<Vec<b212::TradeProposal>, b212::B212Error> {
        let gov = self.b212_governance()?;
        gov.list_pending().await
    }

    /// Approuve une proposition B212.
    pub async fn b212_approve_proposal(
        &self,
        id: &str,
    ) -> Result<b212::TradeProposal, b212::B212Error> {
        let gov = self.b212_governance()?;
        gov.approve(id).await
    }

    /// Rejette une proposition B212.
    pub async fn b212_reject_proposal(
        &self,
        id: &str,
        reason: &str,
    ) -> Result<b212::TradeProposal, b212::B212Error> {
        let gov = self.b212_governance()?;
        gov.reject(id, reason).await
    }

    fn b212_governance(&self) -> Result<B212GovernanceService, b212::B212Error> {
        let journal = self
            .deps
            .b212_journal
            .clone()
            .ok_or_else(|| b212::B212Error::Config("journal B212 absent".into()))?;
        let proposals = self
            .deps
            .b212_proposals
            .clone()
            .ok_or_else(|| b212::B212Error::Config("proposals B212 absent".into()))?;
        Ok(B212GovernanceService::new(journal, proposals))
    }

    /// Tour agent pour un agent persistant (Phase 2b) — personality + session `agent-{id}`.
    ///
    /// # Errors
    ///
    /// Propage [`AgentError`] si l'agent est introuvable ou si la boucle échoue.
    pub async fn agent_turn_for(
        &self,
        agent_id: &str,
        message: &str,
    ) -> Result<AgentTurnResult, AgentError> {
        let manager = self.agent_manager().await?;
        manager
            .run_turn(agent_id, message, Some(Arc::clone(&self.skills)))
            .await
            .map_err(AgentError::from)
    }

    /// Tour agent persistant avec streaming (gateway Phase 2b).
    pub async fn agent_turn_for_with_stream(
        &self,
        agent_id: &str,
        message: &str,
        stream: AgentStreamSink,
    ) -> Result<AgentTurnResult, AgentError> {
        let manager = self.agent_manager().await?;
        manager
            .run_turn_with_stream(agent_id, message, Some(Arc::clone(&self.skills)), stream)
            .await
            .map_err(AgentError::from)
    }

    /// Tour agent complet Phase 7 — contexte graphe + outils mémoire + session.
    ///
    /// # Errors
    ///
    /// Propage [`AgentError`] si la boucle agent échoue.
    pub async fn agent_turn(
        &self,
        session_key: SessionKey,
        message: &str,
    ) -> Result<AgentTurnResult, AgentError> {
        let config = AgentConfig::from_settings(&self.deps.config.agent);
        let agent = AgentLoop::new(self.deps.clone(), config, Some(Arc::clone(&self.skills)));
        agent
            .run_turn(AgentTurnRequest {
                session_key,
                message: message.to_string(),
                personality_prefix: None,
            })
            .await
    }

    /// Tour agent avec configuration personnalisée.
    ///
    /// # Errors
    ///
    /// Propage [`AgentError`] si la boucle agent échoue.
    pub async fn agent_turn_with_config(
        &self,
        session_key: SessionKey,
        message: &str,
        config: AgentConfig,
    ) -> Result<AgentTurnResult, AgentError> {
        let agent = AgentLoop::new(self.deps.clone(), config, Some(Arc::clone(&self.skills)));
        agent
            .run_turn(AgentTurnRequest {
                session_key,
                message: message.to_string(),
                personality_prefix: None,
            })
            .await
    }

    /// Tour agent avec streaming d'événements (gateway Phase 8).
    ///
    /// # Errors
    ///
    /// Propage [`AgentError`] si la boucle agent échoue.
    pub async fn agent_turn_with_stream(
        &self,
        session_key: SessionKey,
        message: &str,
        stream: AgentStreamSink,
    ) -> Result<AgentTurnResult, AgentError> {
        let config = AgentConfig::from_settings(&self.deps.config.agent);
        let agent = AgentLoop::new(self.deps.clone(), config, Some(Arc::clone(&self.skills)));
        agent
            .run_turn_with_stream(
                AgentTurnRequest {
                    session_key,
                    message: message.to_string(),
                    personality_prefix: None,
                },
                stream,
            )
            .await
    }

    /// Chat libre avec le provider LLM configuré (sans boucle agent).
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
        let draft = MemoryDraft::new("Nouveau", "Souvenir assimilé.");
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
        assert!(skills.iter().any(|entry| entry.name == "noop"));
        let out = f
            .execute_skill("noop", &SkillContext::default())
            .await
            .unwrap();
        assert_eq!(out.message, "noop ok");
    }

    #[tokio::test]
    async fn facade_agent_turn_returns_reply() {
        let f = facade();
        let config = AgentConfig {
            message_preprocess: false,
            ..AgentConfig::default()
        };
        let result = f
            .agent_turn_with_config(SessionKey::default_chat(), "Salut agent", config)
            .await
            .unwrap();
        assert_eq!(result.reply, "Salut agent");
        assert_eq!(result.session_key, SessionKey::default_chat());
    }

    #[tokio::test]
    async fn facade_executes_operational_skills() {
        let f = facade();
        let skills = f.list_skills();
        assert!(skills.iter().any(|entry| entry.name == "list_memories"));
        assert!(skills.iter().any(|entry| entry.name == "search"));
        assert!(skills.iter().any(|entry| entry.name == "assimilate"));

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
