use cortex::{MemoryDraft, SearchFilter};

use crate::deps::AppDependencies;
use crate::error::OrchestratorError;
use crate::memory::{
    build_insight_user_prompt, generate_insight_draft, is_duplicate_draft,
    INSIGHT_ASSIMILATION_SYSTEM_PROMPT,
};
use crate::use_cases::SearchMemories;

/// Use case : extrait un [`MemoryDraft`] via le pipeline insight (sans persistance).
pub struct GenerateInsightDraft {
    deps: AppDependencies,
}

impl GenerateInsightDraft {
    /// Crée le use case avec les dépendances injectées.
    #[must_use]
    pub fn new(deps: AppDependencies) -> Self {
        Self { deps }
    }

    /// Génère un brouillon typé depuis du texte brut (filtre skip + dédup Jaccard).
    ///
    /// # Errors
    ///
    /// Propage [`OrchestratorError`] si le LLM échoue ou si le brouillon est dupliqué / ignoré.
    pub async fn execute(
        &self,
        text: &str,
        tags: &[String],
        source_session: Option<&str>,
        system_prompt: Option<&str>,
    ) -> Result<MemoryDraft, OrchestratorError> {
        let system = system_prompt.unwrap_or(INSIGHT_ASSIMILATION_SYSTEM_PROMPT);
        let related = self.fetch_related_context(text).await?;
        let user_prompt = build_insight_user_prompt(text, &related, tags);

        let mut draft = match generate_insight_draft(self.deps.llm.as_ref(), system, &user_prompt)
            .await
            .map_err(OrchestratorError::from)?
        {
            Some(d) => d,
            None => {
                return Err(OrchestratorError::InsightSkipped {
                    reason: "contribution non significative".into(),
                });
            }
        };

        if let Some(path) = source_session {
            if !draft.source_files.contains(&path.to_string()) {
                draft.source_files.push(path.to_string());
            }
        }

        if self.is_duplicate(&draft).await? {
            return Err(OrchestratorError::InsightSkipped {
                reason: "brouillon dupliqué (Jaccard)".into(),
            });
        }

        if let Some(usage) = self.deps.llm.last_usage() {
            self.deps.events.publish_llm_usage(&usage);
        }

        tracing::info!(
            title = %draft.title,
            kind = ?draft.kind,
            "MemoryDraft généré (sans assimilation)"
        );
        Ok(draft)
    }

    async fn fetch_related_context(&self, text: &str) -> Result<String, OrchestratorError> {
        let limit = self.deps.config.memory.insight_related_search_limit;
        let filter = SearchFilter {
            limit: Some(limit),
            ..SearchFilter::default()
        };
        let query = text.chars().take(500).collect::<String>();
        let hits = SearchMemories::new(self.deps.clone())
            .execute(&query, &filter)
            .await?;
        if hits.is_empty() {
            return Ok(String::new());
        }
        let mut lines = Vec::new();
        for hit in &hits {
            if let Ok(memory) = self.deps.memory_repo.get_by_id(hit.memory_id).await {
                lines.push(format!("- {} (score {:.2})", memory.title, hit.score));
            }
        }
        Ok(lines.join("\n"))
    }

    async fn is_duplicate(&self, draft: &MemoryDraft) -> Result<bool, OrchestratorError> {
        let threshold = self.deps.config.memory.dedup_jaccard_threshold;
        use crate::draft::DraftStatus;

        let pending: Vec<MemoryDraft> = self
            .deps
            .draft_repo
            .list(Some(DraftStatus::Pending))
            .await?
            .into_iter()
            .map(|s| s.draft)
            .collect();
        let existing_memories: Vec<MemoryDraft> = self
            .deps
            .memory_repo
            .list()
            .await?
            .into_iter()
            .map(|m| MemoryDraft {
                title: m.title,
                content: m.content,
                tags: m.tags.iter().map(|t| t.as_str().to_string()).collect(),
                backlinks: vec![],
                kind: m.kind,
                structured: m.structured,
                source_files: vec![],
            })
            .collect();
        let mut pool = pending;
        pool.extend(existing_memories);
        Ok(is_duplicate_draft(draft, &pool, threshold))
    }
}