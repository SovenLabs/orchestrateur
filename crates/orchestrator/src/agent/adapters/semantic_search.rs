//! Adapter [`SemanticSearch`] — recherche sémantique enrichie pour l'agent.

use async_trait::async_trait;

use cortex::{ContextSearchHit, RetrievalError, SearchFilter, SemanticSearch};

use crate::deps::AppDependencies;
use crate::error::OrchestratorError;
use crate::use_cases::SearchMemories;

/// Recherche sémantique haut niveau (embedding + vector store + chargement mémoires).
pub struct CortexSemanticSearch {
    deps: AppDependencies,
}

impl CortexSemanticSearch {
    /// Crée l'adapter avec les dépendances injectées.
    #[must_use]
    pub fn new(deps: AppDependencies) -> Self {
        Self { deps }
    }

    fn map_error(err: OrchestratorError) -> RetrievalError {
        match err {
            OrchestratorError::Embedding(e) => RetrievalError::EmbeddingFailed(e),
            OrchestratorError::Cortex(e) => RetrievalError::Cortex(e),
            OrchestratorError::Security(_) => RetrievalError::VectorStoreUnavailable,
            _ => RetrievalError::VectorStoreUnavailable,
        }
    }
}

#[async_trait]
impl SemanticSearch for CortexSemanticSearch {
    async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ContextSearchHit>, RetrievalError> {
        let filter = SearchFilter {
            limit: Some(limit.max(1)),
            ..SearchFilter::default()
        };
        let hits = SearchMemories::new(self.deps.clone())
            .execute(query, &filter)
            .await
            .map_err(Self::map_error)?;

        if hits.is_empty() {
            return Err(RetrievalError::NoRelevantMemories);
        }

        let mut enriched = Vec::with_capacity(hits.len());
        for hit in hits {
            let memory = self
                .deps
                .memory_repo
                .get_by_id(hit.memory_id)
                .await
                .map_err(RetrievalError::Cortex)?;
            enriched.push(ContextSearchHit {
                memory,
                score: hit.score,
            });
        }
        Ok(enriched)
    }
}

impl CortexSemanticSearch {
    /// Recherche tolérante — retourne une liste vide au lieu d'erreur si aucun hit.
    pub async fn search_or_empty(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ContextSearchHit>, RetrievalError> {
        match self.search(query, limit).await {
            Ok(h) => Ok(h),
            Err(RetrievalError::NoRelevantMemories) => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }
}