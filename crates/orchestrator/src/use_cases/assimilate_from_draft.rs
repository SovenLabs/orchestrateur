use cortex::{
    BacklinkCalculator, DomainEvent, KnowledgeGraph, Memory, MemoryId,
};

use crate::deps::AppDependencies;
use crate::error::OrchestratorError;
use crate::memory_draft::MemoryDraft;

/// Résultat d'une assimilation dry-run : mémoire persistée + événements émis.
pub type AssimilationResult = (Memory, Vec<DomainEvent>);

/// Use case central Phase 2 : assimile un brouillon sans appel IA.
pub struct AssimilateFromDraft {
    deps: AppDependencies,
}

impl AssimilateFromDraft {
    /// Crée le use case avec les dépendances injectées.
    #[must_use]
    pub fn new(deps: AppDependencies) -> Self {
        Self { deps }
    }

    /// Exécute le flux d'assimilation complet en dry-run.
    ///
    /// # Errors
    ///
    /// Propage une [`OrchestratorError`] si la validation, le graphe ou la persistance échoue.
    pub async fn execute(&self, draft: MemoryDraft) -> Result<AssimilationResult, OrchestratorError> {
        let mut memory = draft.into_memory()?;
        let draft_backlinks = memory.backlinks.clone();

        tracing::info!(title = %memory.title, memory_id = %memory.id, "assimilation démarrée");

        let embedding = self
            .deps
            .embedding
            .embed(&format!("{} {}", memory.title, memory.content))
            .await?;

        let corpus = self.build_embedding_corpus(memory.id).await?;
        let semantic = BacklinkCalculator::compute_semantic_backlinks(
            memory.id,
            &embedding,
            &corpus,
            self.deps.config.similarity_thresholds,
        )?;

        let wikilinks = BacklinkCalculator::extract_wikilinks(&memory.content);
        let computed = BacklinkCalculator::merge_backlinks(semantic, wikilinks)?;
        let final_backlinks = BacklinkCalculator::union_backlinks(computed, draft_backlinks);
        BacklinkCalculator::apply_to_memory(&mut memory, final_backlinks);

        self.deps.memory_repo.save(&memory).await?;
        self.deps
            .vector_store
            .upsert(memory.id, &embedding)
            .await?;

        let all_memories = self.deps.memory_repo.list().await?;
        let graph = KnowledgeGraph::from_memories(&all_memories);
        graph.validate()?;
        graph.validate_resolvable(&all_memories)?;

        let events = vec![
            DomainEvent::memory_assimilated(memory.id, memory.backlink_count()),
            DomainEvent::knowledge_graph_validated(graph.node_count(), graph.edge_count()),
        ];
        self.deps.events.publish(&events);

        tracing::info!(
            memory_id = %memory.id,
            backlinks = memory.backlink_count(),
            nodes = graph.node_count(),
            "assimilation terminée"
        );

        Ok((memory, events))
    }

    async fn build_embedding_corpus(
        &self,
        exclude_id: MemoryId,
    ) -> Result<Vec<(MemoryId, Vec<f32>)>, OrchestratorError> {
        let existing = self.deps.memory_repo.list().await?;
        let mut corpus = Vec::new();

        for mem in existing {
            if mem.id == exclude_id {
                continue;
            }
            let embedding = if let Some(cached) = self.deps.vector_store.get_embedding(mem.id).await? {
                tracing::debug!(memory_id = %mem.id, "embedding récupéré du cache vectoriel");
                cached
            } else {
                let text = format!("{} {}", mem.title, mem.content);
                tracing::debug!(memory_id = %mem.id, "embedding calculé (cache absent)");
                self.deps.embedding.embed(&text).await?
            };
            corpus.push((mem.id, embedding));
        }

        Ok(corpus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_draft::{BacklinkDraft, BacklinkDraftKind, MemoryDraft};
    use crate::testing::MockBundle;
    use crate::use_cases::save_memory::SaveMemory;
    use cortex::{DomainEvent, Memory, MemoryRepository};

    #[tokio::test]
    async fn assimilates_valid_draft_and_emits_event() {
        let bundle = MockBundle::new();
        let deps = bundle.into_deps();

        let existing = Memory::new("Architecture", "Le Cortex est prioritaire.").unwrap();
        deps.memory_repo.save(&existing).await.unwrap();

        let draft = MemoryDraft {
            title: "Décision Phase 2".into(),
            content: "L'orchestrateur pilote le Cortex.".into(),
            tags: vec!["architecture".into()],
            backlinks: vec![],
        };

        let (memory, events) = AssimilateFromDraft::new(deps).execute(draft).await.unwrap();
        assert_eq!(memory.title, "Décision Phase 2");
        assert_eq!(memory.tags.len(), 1);
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], DomainEvent::MemoryAssimilated(_)));
        assert!(matches!(events[1], DomainEvent::KnowledgeGraphValidated(_)));
    }

    #[tokio::test]
    async fn assimilates_with_semantic_backlinks() {
        let bundle = MockBundle::new();
        let deps = bundle.into_deps();

        let related = Memory::new(
            "Rust patterns",
            "Patterns pour un code maintenable sur 10 ans.",
        )
        .unwrap();
        SaveMemory::new(deps.clone())
            .execute(&related)
            .await
            .unwrap();

        let draft = MemoryDraft {
            title: "Maintenabilité".into(),
            content: "Patterns pour un code maintenable sur 10 ans.".into(),
            tags: vec![],
            backlinks: vec![],
        };

        let (memory, _) = AssimilateFromDraft::new(deps).execute(draft).await.unwrap();
        assert!(
            memory.backlinks.iter().any(|bl| bl.target == related.id),
            "devrait créer un backlink sémantique vers la mémoire similaire"
        );
    }

    #[tokio::test]
    async fn preserves_draft_backlinks_not_overwritten() {
        let mut bundle = MockBundle::new();
        bundle.config.similarity_thresholds.semantic_min = 0.99;
        let deps = bundle.into_deps();

        let other = Memory::new("Autre", "Sans rapport.").unwrap();
        let other_id = other.id;
        deps.memory_repo.save(&other).await.unwrap();

        let draft = MemoryDraft {
            title: "Avec lien LLM".into(),
            content: "Contenu sans similarité.".into(),
            tags: vec![],
            backlinks: vec![BacklinkDraft {
                target: other_id.to_string(),
                score: 0.42,
                kind: BacklinkDraftKind::Semantic,
            }],
        };

        let (memory, _) = AssimilateFromDraft::new(deps).execute(draft).await.unwrap();
        let bl = memory
            .backlinks
            .iter()
            .find(|bl| bl.target == other_id)
            .expect("backlink draft conservé");
        assert!((bl.score - 0.42).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn rejects_invalid_draft() {
        let bundle = MockBundle::new();
        let draft = MemoryDraft {
            title: "".into(),
            content: "x".into(),
            tags: vec![],
            backlinks: vec![],
        };
        let err = AssimilateFromDraft::new(bundle.into_deps())
            .execute(draft)
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            OrchestratorError::Cortex(cortex::CortexError::EmptyTitle)
        ));
    }

    #[tokio::test]
    async fn assimilates_explicit_wikilink() {
        let bundle = MockBundle::new();
        let deps = bundle.into_deps();

        let target = Memory::new("Cible", "Mémoire cible.").unwrap();
        let target_id = target.id;
        deps.memory_repo.save(&target).await.unwrap();

        let draft = MemoryDraft {
            title: "Référence".into(),
            content: format!("Voir [[{target_id}]] pour détails."),
            tags: vec![],
            backlinks: vec![BacklinkDraft {
                target: target_id.to_string(),
                score: 0.5,
                kind: BacklinkDraftKind::ExplicitWikilink,
            }],
        };

        let (memory, _) = AssimilateFromDraft::new(deps).execute(draft).await.unwrap();
        assert!(memory.backlinks.iter().any(|bl| bl.target == target_id));
    }

    #[tokio::test]
    async fn rejects_backlink_to_missing_node_in_graph() {
        let bundle = MockBundle::new();
        let deps = bundle.into_deps();
        let ghost = MemoryId::new();

        let draft = MemoryDraft {
            title: "Lien fantôme".into(),
            content: format!("Réf [[{ghost}]]"),
            tags: vec![],
            backlinks: vec![],
        };

        let err = AssimilateFromDraft::new(deps).execute(draft).await.unwrap_err();
        assert!(matches!(
            err,
            OrchestratorError::Cortex(cortex::CortexError::MemoryNotFound(_))
        ));
    }
}