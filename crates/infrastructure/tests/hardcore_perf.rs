//! Problème 1 — Performance et scalabilité (LanceDB + embeddings)

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::time::Instant;

use cortex::{EmbeddingProvider, SearchFilter, VectorStore};
use infrastructure::{FileMemoryRepository, LancedbVectorStore};
use orchestrator::testing::{percentile_ms, test_memory, InMemoryEmbeddingProvider};
use orchestrator::{AppDependencies, OrchestratorConfig, OrchestratorFacade};
use tempfile::tempdir;

const EMBED_DIM: usize = 16;

async fn setup_lancedb_workspace() -> (tempfile::TempDir, AppDependencies) {
    let dir = tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("memories")).expect("memories dir");

    let mut cfg = OrchestratorConfig::default();
    cfg.workspace_root = dir.path().to_path_buf();
    cfg.embedding_dim = EMBED_DIM;
    cfg.vector_store.store_type = "lancedb".into();
    cfg.vector_store.embedding_dimension = EMBED_DIM;

    let vector_store: Arc<dyn VectorStore> = Arc::new(
        LancedbVectorStore::open(dir.path().join(".orchestrateur/lancedb"), EMBED_DIM)
            .await
            .expect("lancedb"),
    );
    let embedding: Arc<dyn cortex::EmbeddingProvider> =
        Arc::new(InMemoryEmbeddingProvider::new(EMBED_DIM));
    let memory_repo: Arc<dyn cortex::MemoryRepository> =
        Arc::new(FileMemoryRepository::new(cfg.memories_dir()));
    let llm = Arc::new(orchestrator::testing::InMemoryLlmProvider);

    let session_repo: Arc<dyn cortex::SessionRepository> =
        Arc::new(orchestrator::testing::InMemorySessionRepository::new());
    let deps = AppDependencies::for_tests(
        memory_repo,
        vector_store,
        embedding,
        llm,
        session_repo,
        cfg,
        Arc::new(orchestrator::NoopEventPublisher),
    );
    (dir, deps)
}

#[tokio::test]
async fn intensity1_search_100_memories_50_queries() {
    let (_dir, deps) = setup_lancedb_workspace().await;
    let facade = OrchestratorFacade::new(deps.clone());
    let provider = InMemoryEmbeddingProvider::new(EMBED_DIM);

    for i in 0..100 {
        let mem = test_memory(i).expect("memory");
        facade.save_memory(&mem).await.expect("save");
    }

    let query_emb = provider
        .embed("recherche sémantique test")
        .await
        .expect("embed");
    let mut latencies = Vec::with_capacity(50);

    for _ in 0..50 {
        let start = Instant::now();
        let _ = deps
            .vector_store
            .semantic_search(query_emb.as_slice(), 10)
            .await
            .expect("search");
        latencies.push(start.elapsed().as_millis());
    }

    let total: u128 = latencies.iter().sum();
    let p95 = percentile_ms(latencies, 95);

    assert!(
        total < 8_000,
        "50 recherches sur 100 mémoires : total {total}ms > 8000ms"
    );
    assert!(
        p95 < 200,
        "p95 recherche {p95}ms — seuil debug 200ms (release plus strict)"
    );

    let hits = facade
        .search_memories("sémantique", &SearchFilter::default())
        .await
        .expect("facade search");
    assert!(!hits.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 12)]
#[ignore = "hardcore intensity 2 — 2000 mémoires + charge concurrente"]
async fn intensity2_concurrent_2000_memories() {
    let (_dir, deps) = setup_lancedb_workspace().await;
    let store = Arc::clone(&deps.vector_store);
    let facade = Arc::new(OrchestratorFacade::new(deps));
    let provider = InMemoryEmbeddingProvider::new(EMBED_DIM);

    for i in 0..2000 {
        let mem = test_memory(i).expect("memory");
        facade.save_memory(&mem).await.expect("save");
    }

    let query_emb = provider.embed("charge").await.expect("embed");
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let store = Arc::clone(&store);
            let q = query_emb.as_slice().to_vec();
            tokio::spawn(async move {
                let mut local = Vec::new();
                for _ in 0..50 {
                    let start = Instant::now();
                    let _ = store.semantic_search(&q, 10).await.expect("search");
                    local.push(start.elapsed().as_millis());
                }
                local
            })
        })
        .collect();

    let mut all = Vec::new();
    for h in handles {
        all.extend(h.await.expect("join"));
    }
    let p95 = percentile_ms(all, 95);
    assert!(p95 < 500, "p95 concurrent {p95}ms");
}

#[tokio::test]
async fn intensity1_embed_batch_used_in_assimilation_corpus() {
    use orchestrator::memory_draft::MemoryDraft;
    use orchestrator::testing::CountingEmbeddingProvider;
    let bundle = orchestrator::testing::MockBundle::new();
    let counting = CountingEmbeddingProvider::new(bundle.config.embedding_dim);
    let deps = AppDependencies::for_tests(
        bundle.memory_repo.clone(),
        bundle.vector_store.clone(),
        counting.clone(),
        bundle.llm,
        bundle.session_repo,
        bundle.config,
        Arc::new(orchestrator::NoopEventPublisher),
    );

    for i in 0..5 {
        let mem = test_memory(i).expect("mem");
        deps.memory_repo.save(&mem).await.expect("repo seul");
    }

    let draft = MemoryDraft {
        title: "Batch test".into(),
        content: "Vérifie embed_batch dans le corpus.".into(),
        tags: vec![],
        backlinks: vec![],
    };

    OrchestratorFacade::new(deps)
        .assimilate_from_draft(draft)
        .await
        .expect("assimilation");

    assert!(
        counting.batch_calls() >= 1,
        "embed_batch doit être appelé pour le corpus (appels: {})",
        counting.batch_calls()
    );
}
