//! Adapter [`ContextProvider`] — contexte graphe + recherche + historique session.

use async_trait::async_trait;

use cortex::{
    AgentContext, ContextProvider, KnowledgeGraph, Memory, RetrievalError, SessionKey,
};

use crate::agent::AgentConfig;
use crate::deps::AppDependencies;
use crate::error::OrchestratorError;
use crate::use_cases::ListMemories;

use super::semantic_search::CortexSemanticSearch;

/// Construit un [`AgentContext`] à partir des ports Cortex.
pub struct CortexContextProvider {
    deps: AppDependencies,
    config: AgentConfig,
}

impl CortexContextProvider {
    /// Crée le provider avec config agent (graphe, limites).
    #[must_use]
    pub fn new(deps: AppDependencies, config: AgentConfig) -> Self {
        Self { deps, config }
    }

    async fn graph_context(&self) -> Result<Option<String>, OrchestratorError> {
        if !self.config.graph_context_enabled {
            return Ok(None);
        }
        let memories = ListMemories::new(self.deps.clone()).execute().await?;
        if memories.is_empty() {
            return Ok(None);
        }

        let graph = KnowledgeGraph::from_memories(&memories);
        let title_by_id: std::collections::HashMap<_, _> = memories
            .iter()
            .map(|m: &Memory| (m.id, m.title.as_str()))
            .collect();

        let hubs = graph
            .hub_ranking()
            .into_iter()
            .take(self.config.graph_hub_limit)
            .map(|(id, inbound)| {
                let title = title_by_id
                    .get(&id)
                    .map_or_else(|| id.to_string(), |t| (*t).to_string());
                format!("- {title} ({inbound} liens entrants)")
            })
            .collect::<Vec<_>>();

        Ok(Some(format!(
            "Nœuds: {}, arêtes: {}\nHubs:\n{}",
            graph.node_count(),
            graph.edge_count(),
            hubs.join("\n")
        )))
    }

    fn map_error(err: OrchestratorError) -> RetrievalError {
        match err {
            OrchestratorError::Cortex(e) => RetrievalError::Cortex(e),
            OrchestratorError::Embedding(e) => RetrievalError::EmbeddingFailed(e),
            _ => RetrievalError::VectorStoreUnavailable,
        }
    }
}

#[async_trait]
impl ContextProvider for CortexContextProvider {
    async fn build_context(
        &self,
        query: &str,
        session_id: Option<SessionKey>,
        limit: usize,
    ) -> Result<AgentContext, RetrievalError> {
        let graph_context = self.graph_context().await.map_err(Self::map_error)?;

        let search = CortexSemanticSearch::new(self.deps.clone());
        let hits = if self.config.proactive_memory_search {
            search
                .search_or_empty(query, limit.max(1))
                .await?
        } else {
            Vec::new()
        };
        let memories: Vec<Memory> = hits.into_iter().map(|h| h.memory).collect();

        let session_turns = if let Some(key) = session_id {
            let turns = self
                .deps
                .session_repo
                .list_turns(&key)
                .await
                .map_err(RetrievalError::Cortex)?;
            let start = turns.len().saturating_sub(self.config.max_history_turns);
            turns[start..].to_vec()
        } else {
            Vec::new()
        };

        Ok(AgentContext {
            memories,
            graph_context,
            session_turns,
        })
    }
}