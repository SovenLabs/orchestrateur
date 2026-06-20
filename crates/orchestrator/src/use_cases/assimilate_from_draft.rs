use cortex::{BacklinkCalculator, DomainEvent, Memory, MemoryId};

use crate::deps::AppDependencies;
use crate::error::OrchestratorError;
use crate::memory_draft::MemoryDraft;

/// Résultat d'une assimilation dry-run : mémoire persistée + événements émis.
pub type AssimilationResult = (Memory, Vec<DomainEvent>);

/// Use case central Phase 2 : assimile un brouillon sans appel IA.
///
/// Flux :
/// 1. `MemoryDraft` → validation domaine (`into_memory`)
/// 2. Calcul embeddings + backlinks via `BacklinkCalculator`
/// 3. Persistance via ports
/// 4. Émission `DomainEvent::MemoryAssimilated`
pub struct AssimilateFromDraft {
    deps: AppDependencies,
}

impl AssimilateFromDraft {
    /// Crée le use case avec les dépendances injectées.
    pub fn new(deps: AppDependencies) -> Self {
        Self { deps }
    }

    /// Exécute le flux d'assimilation complet en dry-run.
    pub async fn execute(&self, draft: MemoryDraft) -> Result<AssimilationResult, OrchestratorError> {
        let mut memory = draft.into_memory()?;

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
        let backlinks = BacklinkCalculator::merge_backlinks(semantic, wikilinks)?;
        BacklinkCalculator::apply_to_memory(&mut memory, backlinks);

        self.deps.memory_repo.save(&memory).await?;
        self.deps
            .vector_store
            .upsert(memory.id, &embedding)
            .await?;

        let events = vec![DomainEvent::memory_assimilated(
            memory.id,
            memory.backlink_count(),
        )];

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
            let text = format!("{} {}", mem.title, mem.content);
            let embedding = self.deps.embedding.embed(&text).await?;
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
    use cortex::{DomainEvent, Memory};

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
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::MemoryAssimilated(e) => {
                assert_eq!(e.memory_id, memory.id);
            }
        }
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
        deps.memory_repo.save(&related).await.unwrap();

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
}