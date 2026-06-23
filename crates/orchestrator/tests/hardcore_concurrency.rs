//! Problème 4 — Concurrence et conditions de course

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use orchestrator::memory_draft::MemoryDraft;
use orchestrator::testing::{assert_workspace_consistent, build_test_facade, MockBundle};
use orchestrator::OrchestratorFacade;

fn shared_facade() -> Arc<OrchestratorFacade> {
    Arc::new(build_test_facade(MockBundle::new().into_deps()))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "charge: 10 assimilations parallèles"]
async fn intensity1_ten_parallel_assimilations() {
    let facade = shared_facade();
    let deps = facade.deps().clone();

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let f = facade.clone();
            tokio::spawn(async move {
                let draft = MemoryDraft::new(
                    format!("Parallèle {i}"),
                    format!("Assimilation concurrente {i}."),
                );
                f.assimilate_from_draft(draft).await
            })
        })
        .collect();

    let mut ok = 0usize;
    for handle in handles {
        if handle.await.expect("join").is_ok() {
            ok += 1;
        }
    }

    assert_eq!(
        ok, 10,
        "toutes les assimilations parallèles doivent réussir"
    );
    assert_workspace_consistent(&deps)
        .await
        .expect("état cohérent après concurrence");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[ignore = "charge: 50 assimilations + 200 recherches mixtes"]
async fn intensity2_mixed_assimilations_and_searches() {
    let facade = shared_facade();
    let deps = facade.deps().clone();

    let assimilate_handles: Vec<_> = (0..50)
        .map(|i| {
            let f = facade.clone();
            tokio::spawn(async move {
                let draft = MemoryDraft::new(
                    format!("Mix {i}"),
                    format!("Recherche et assimilation {i}."),
                );
                f.assimilate_from_draft(draft).await
            })
        })
        .collect();

    let search_handles: Vec<_> = (0..200)
        .map(|i| {
            let f = facade.clone();
            tokio::spawn(async move {
                let query = format!("assimilation {i}");
                f.search_memories(&query, &cortex::SearchFilter::default())
                    .await
            })
        })
        .collect();

    for h in assimilate_handles {
        h.await.expect("join").expect("assimilation");
    }
    for h in search_handles {
        let _ = h.await.expect("join").expect("recherche");
    }

    assert_workspace_consistent(&deps)
        .await
        .expect("cohérence après charge mixte");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 16)]
#[ignore = "hardcore intensity 3 — 200 assimilations simultanées"]
async fn intensity3_burst_assimilations() {
    let facade = shared_facade();
    let handles: Vec<_> = (0..200)
        .map(|i| {
            let f = facade.clone();
            tokio::spawn(async move {
                let draft = MemoryDraft::new(format!("Burst {i}"), "Charge extrême.");
                f.assimilate_from_draft(draft).await
            })
        })
        .collect();

    let mut ok = 0;
    for h in handles {
        if h.await.expect("join").is_ok() {
            ok += 1;
        }
    }
    assert!(ok >= 190, "taux de succès > 95% attendu, obtenu {ok}/200");
}
