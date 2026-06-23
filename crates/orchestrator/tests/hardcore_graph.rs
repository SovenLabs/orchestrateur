//! Problème 3 — Cohérence du graphe de connaissances

#![allow(clippy::unwrap_used, clippy::expect_used)]

use orchestrator::memory_draft::{BacklinkDraft, BacklinkDraftKind, MemoryDraft};
use orchestrator::testing::{
    assert_no_ghost_nodes, assert_workspace_consistent, build_test_facade, test_memory, MockBundle,
};
#[tokio::test]
#[ignore = "sécurité: rejet backlink vers nœud fantôme"]
async fn intensity1_invalid_draft_backlinks_rejected() {
    let bundle = MockBundle::new();
    let deps = bundle.into_deps();
    let ghost_id = cortex::MemoryId::new();

    let mut draft = MemoryDraft::new("Lien invalide", "Sans wikilink résolvable.");
    draft.backlinks = vec![BacklinkDraft {
        target: ghost_id.to_string(),
        score: 0.9,
        kind: BacklinkDraftKind::Semantic,
    }];

    let err = build_test_facade(deps)
        .assimilate_from_draft(draft)
        .await
        .expect_err("backlink vers nœud inexistant doit échouer");

    assert!(matches!(
        err,
        orchestrator::OrchestratorError::Cortex(cortex::CortexError::MemoryNotFound(_))
    ));
}

#[tokio::test]
#[ignore = "charge: cohérence graphe après 30 opérations"]
async fn intensity1_workspace_consistent_after_assimilations() {
    let bundle = MockBundle::new();
    let deps = bundle.into_deps();
    let facade = build_test_facade(deps.clone());

    for i in 0..20 {
        let mem = test_memory(i).expect("mémoire valide");
        facade.save_memory(&mem).await.expect("save");
    }

    for i in 0..10 {
        let draft = MemoryDraft::new(
            format!("Assimilation {i}"),
            format!("Contenu assimilé numéro {i}."),
        );
        facade
            .assimilate_from_draft(draft)
            .await
            .expect("assimilation");
    }

    assert_workspace_consistent(&deps)
        .await
        .expect("cohérence workspace");
    assert_no_ghost_nodes(&deps).await.expect("pas de fantômes");
}

#[tokio::test]
#[ignore = "hardcore intensity 2 — 500 assimilations avec chevauchement"]
async fn intensity2_mass_overlap_assimilations() {
    let bundle = MockBundle::new();
    let deps = bundle.into_deps();
    let facade = build_test_facade(deps.clone());

    let seed = test_memory(0).expect("seed");
    facade.save_memory(&seed).await.expect("save seed");

    for i in 0..500 {
        let mut draft = MemoryDraft::new(
            format!("Overlap {i}"),
            "Le Cortex est le squelette durable.",
        );
        draft.tags = vec!["architecture".into()];
        draft.backlinks = vec![BacklinkDraft {
            target: seed.id.to_string(),
            score: 0.5,
            kind: BacklinkDraftKind::Semantic,
        }];
        facade.assimilate_from_draft(draft).await.expect("assim");
    }

    assert_workspace_consistent(&deps)
        .await
        .expect("graphe cohérent après 500 assimilations");
}
