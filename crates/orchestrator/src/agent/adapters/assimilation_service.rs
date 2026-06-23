//! Adapter [`AssimilationService`] — assimilation intelligente des tours agent.

use async_trait::async_trait;

use cortex::{
    AssimilationError, AssimilationPolicy, AssimilationResult, AssimilationService,
    ConversationTurn,
};

use crate::agent::AgentConfig;
use crate::deps::AppDependencies;
use crate::error::OrchestratorError;

use crate::use_cases::{AssimilateFromText, GenerateInsightDraft};

use super::change_detector::{parse_agent_exchange, ChangeDetector, ChangeDetectorConfig};
use super::semantic_search::CortexSemanticSearch;

/// Assimilation des échanges agent selon [`AssimilationPolicy`].
pub struct CortexAssimilationService {
    deps: AppDependencies,
    detector: ChangeDetector,
}

impl CortexAssimilationService {
    /// Crée le service avec détection de changement par défaut.
    #[must_use]
    pub fn new(deps: AppDependencies, _config: AgentConfig) -> Self {
        let mut detector_cfg = ChangeDetectorConfig::default();
        detector_cfg.max_redundancy_score = deps
            .config
            .similarity_thresholds
            .semantic_min
            .max(0.75)
            .min(0.92);
        Self {
            deps,
            detector: ChangeDetector::new(detector_cfg),
        }
    }

    fn summary_text(user: &str, assistant: &str) -> String {
        format!(
            "Échange agent — utilisateur: {user}\nassistant: {assistant}"
        )
    }

    fn map_orchestrator_err(err: OrchestratorError) -> AssimilationError {
        match err {
            OrchestratorError::Validation(v) => AssimilationError::ValidationFailed(vec![
                cortex::ValidationWarning {
                    code: "validation".into(),
                    message: v.to_string(),
                    risk_delta: 1.0,
                },
            ]),
            OrchestratorError::Cortex(e) => AssimilationError::PersistenceFailed(e),
            OrchestratorError::InsightSkipped { .. } => {
                AssimilationError::ChangeDetectionFailed
            }
            OrchestratorError::Llm(_) | OrchestratorError::Embedding(_) => {
                AssimilationError::ChangeDetectionFailed
            }
            OrchestratorError::Security(_) | OrchestratorError::Internal(_) => {
                AssimilationError::ChangeDetectionFailed
            }
            OrchestratorError::Draft(e) => AssimilationError::PersistenceFailed(
                cortex::CortexError::GraphError(e.to_string()),
            ),
        }
    }
}

#[async_trait]
impl AssimilationService for CortexAssimilationService {
    async fn assimilate_turn(
        &self,
        turn: &ConversationTurn,
        policy: AssimilationPolicy,
    ) -> Result<AssimilationResult, AssimilationError> {
        let (user, assistant) = parse_agent_exchange(turn)?;
        let summary = Self::summary_text(&user, &assistant);

        match policy {
            AssimilationPolicy::AutoIfChange => {
                let search = CortexSemanticSearch::new(self.deps.clone());
                if !self
                    .detector
                    .should_assimilate(&search, &user, &assistant)
                    .await?
                {
                    return Ok(AssimilationResult::empty());
                }
                self.persist_assimilation(&summary).await
            }
            AssimilationPolicy::RequireUserApproval => {
                self.pending_draft(&summary).await
            }
            AssimilationPolicy::AlwaysAuto => self.persist_assimilation(&summary).await,
        }
    }
}

impl CortexAssimilationService {
    async fn persist_assimilation(
        &self,
        summary: &str,
    ) -> Result<AssimilationResult, AssimilationError> {
        match AssimilateFromText::new(self.deps.clone())
            .execute(summary, &["agent-turn".into()], None)
            .await
        {
            Ok((memory, _events)) => Ok(AssimilationResult {
                created: vec![memory.id],
                updated: Vec::new(),
                pending_drafts: Vec::new(),
            }),
            Err(OrchestratorError::InsightSkipped { .. }) => Ok(AssimilationResult::empty()),
            Err(e) => Err(Self::map_orchestrator_err(e)),
        }
    }

    async fn pending_draft(
        &self,
        summary: &str,
    ) -> Result<AssimilationResult, AssimilationError> {
        let draft = GenerateInsightDraft::new(self.deps.clone())
            .execute(summary, &["agent-turn".into()], None, None)
            .await
            .map_err(Self::map_orchestrator_err)?;

        let stored = self
            .deps
            .draft_repo
            .create_pending(draft.clone(), Some("agent-loop-v2".into()))
            .await
            .map_err(|e| {
                AssimilationError::PersistenceFailed(cortex::CortexError::GraphError(e.to_string()))
            })?;

        Err(AssimilationError::UserApprovalRequired(vec![stored.draft]))
    }
}