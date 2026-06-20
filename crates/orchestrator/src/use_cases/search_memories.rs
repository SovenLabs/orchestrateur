use cortex::{SearchFilter, SearchHit};

use crate::deps::AppDependencies;
use crate::error::OrchestratorError;

/// Use case : recherche hybride (vectorielle + filtrage tags via le dépôt).
pub struct SearchMemories {
    deps: AppDependencies,
}

impl SearchMemories {
    /// Crée le use case avec les dépendances injectées.
    pub fn new(deps: AppDependencies) -> Self {
        Self { deps }
    }

    /// Recherche des mémoires similaires à la requête textuelle.
    pub async fn execute(
        &self,
        query: &str,
        filter: &SearchFilter,
    ) -> Result<Vec<SearchHit>, OrchestratorError> {
        let embedding = self.deps.embedding.embed(query).await?;
        let mut hits = self
            .deps
            .vector_store
            .hybrid_search(&embedding, filter)
            .await?;

        if !filter.tags.is_empty() {
            hits = self.filter_by_tags(hits, &filter.tags).await?;
        }

        if let Some(limit) = filter.limit {
            hits.truncate(limit);
        }

        Ok(hits)
    }

    async fn filter_by_tags(
        &self,
        hits: Vec<SearchHit>,
        required_tags: &[cortex::Tag],
    ) -> Result<Vec<SearchHit>, OrchestratorError> {
        let mut filtered = Vec::new();
        for hit in hits {
            let memory = self.deps.memory_repo.get_by_id(hit.memory_id).await?;
            if required_tags.iter().all(|tag| memory.has_tag(tag)) {
                filtered.push(hit);
            }
        }
        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockBundle;
    use crate::use_cases::save_memory::SaveMemory;
    use cortex::{Memory, SearchFilter, Tag};

    #[tokio::test]
    async fn search_finds_similar_memory() {
        let bundle = MockBundle::new();
        let deps = bundle.into_deps();
        let mem = Memory::new("Rust architecture", "Le Cortex est le squelette.").unwrap();
        SaveMemory::new(deps.clone()).execute(&mem).await.unwrap();

        let filter = SearchFilter {
            limit: Some(5),
            ..Default::default()
        };
        let hits = SearchMemories::new(deps)
            .execute("architecture squelette", &filter)
            .await
            .unwrap();
        assert!(!hits.is_empty());
        assert_eq!(hits[0].memory_id, mem.id);
    }

    #[tokio::test]
    async fn search_filters_by_tag() {
        let bundle = MockBundle::new();
        let deps = bundle.into_deps();
        let mut mem = Memory::new("Tagged", "Contenu avec tag rust.").unwrap();
        mem.add_tag(Tag::new("rust").unwrap());
        SaveMemory::new(deps.clone()).execute(&mem).await.unwrap();

        let filter = SearchFilter {
            tags: vec![Tag::new("rust").unwrap()],
            limit: Some(5),
            ..Default::default()
        };
        let hits = SearchMemories::new(deps)
            .execute("rust contenu", &filter)
            .await
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].memory_id, mem.id);
    }
}