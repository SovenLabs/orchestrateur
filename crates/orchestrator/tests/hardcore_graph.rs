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

    let draft = MemoryDraft {
        title: "Lien invalide".into(),
        content: "Sans wikilink résolvable.".into(),
        tags: vec![],
        backlinks: vec![BacklinkDraft {
            target: ghost_id.to_string(),
            score: 0.9,
            kind: BacklinkDraftKind::Semantic,
        }],
    };

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
        let draft = MemoryDraft {
            title: format!("Assimilation {i}"),
            content: format!("Contenu assimilé numéro {i}."),
            tags: vec![],
            backlinks: vec![],
        };
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
        let draft = MemoryDraft {
            title: format!("Overlap {i}"),
            content: "Le Cortex est le squelette durable.".into(),
            tags: vec!["architecture".into()],
            backlinks: vec![BacklinkDraft {
                target: seed.id.to_string(),
                score: 0.5,
                kind: BacklinkDraftKind::Semantic,
            }],
        };
        facade.assimilate_from_draft(draft).await.expect("assim");
    }

    assert_workspace_consistent(&deps)
        .await
        .expect("graphe cohérent après 500 assimilations");
}
