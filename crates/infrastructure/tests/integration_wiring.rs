//! Tests d'intégration Phase 3 — skippés si dépendances externes absentes.
//!
//! ## Test Ollama E2E (`ollama_end_to_end_when_available`)
//!
//! Marqué `#[ignore]` car il nécessite Ollama actif en local.
//!
//! Prérequis :
//! 1. Installer [Ollama](https://ollama.com/) et lancer le daemon (`ollama serve`).
//! 2. Tirer les modèles configurés dans `orchestrator.toml` :
//!    ```text
//!    ollama pull qwen3-embedding:8b
//!    ollama pull qwen3:8b
//!    ```
//! 3. Vérifier l'endpoint : `curl http://127.0.0.1:11434/api/tags`
//!
//! Exécution :
//! ```text
//! cargo test -p infrastructure ollama_end_to_end_when_available -- --ignored
//! ```

use std::sync::Arc;

use cortex::{Memory, SearchFilter, VectorStore};
use infrastructure::{build_app_dependencies, LancedbVectorStore};
use orchestrator::{
    testing::InMemoryVectorStore,
    AppDependencies, OrchestratorConfig, OrchestratorFacade,
};
use tempfile::tempdir;

fn lancedb_config(root: &std::path::Path) -> OrchestratorConfig {
    let mut cfg = OrchestratorConfig::load_workspace(root).unwrap();
    cfg.vector_store.store_type = "lancedb".into();
    cfg.vector_store.embedding_dimension = 8;
    cfg.embedding_dim = 8;
    cfg.providers.primary_llm = "ollama".into();
    cfg.providers.fallback_llm.clear();
    cfg.providers.primary_embedding = "ollama".into();
    cfg
}

#[tokio::test]
async fn lancedb_store_with_file_repo_smoke() {
    use infrastructure::FileMemoryRepository;

    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("memories")).unwrap();
    let mut cfg = lancedb_config(dir.path());
    cfg.embedding_dim = 8;

    let vector_store: Arc<dyn VectorStore> = Arc::new(
        LancedbVectorStore::open(dir.path().join(".orchestrateur/lancedb"), 8)
            .await
            .unwrap(),
    );
    let embedding: Arc<dyn cortex::EmbeddingProvider> =
        Arc::new(orchestrator::testing::InMemoryEmbeddingProvider::new(8));
    let llm = Arc::new(orchestrator::testing::InMemoryLlmProvider);
    let memory_repo: Arc<dyn cortex::MemoryRepository> =
        Arc::new(FileMemoryRepository::new(cfg.memories_dir()));

    let deps = AppDependencies::for_tests(
        memory_repo,
        vector_store,
        embedding,
        llm,
        cfg,
        Arc::new(orchestrator::NoopEventPublisher),
    );
    let facade = OrchestratorFacade::new(deps);
    let mem = Memory::new("Intégration LanceDB", "Test persistance vectorielle.").unwrap();
    facade.save_memory(&mem).await.expect("save");
    let hits = facade
        .search_memories("persistance", &SearchFilter::default())
        .await
        .expect("search");
    assert!(!hits.is_empty());
}

#[tokio::test]
#[ignore = "nécessite Ollama sur 127.0.0.1:11434"]
async fn ollama_end_to_end_when_available() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("memories")).unwrap();
    let cfg = lancedb_config(dir.path());

    let deps = build_app_dependencies(cfg).await.expect("deps");
    let facade = OrchestratorFacade::new(deps);
    let (memory, _) = facade
        .assimilate("Note de test Ollama Phase 3.", None)
        .await
        .expect("assimilate");
    assert!(!memory.title.is_empty());
}

#[tokio::test]
async fn memory_mode_deps_via_mocks() {
    let bundle = orchestrator::testing::MockBundle::new();
    let deps = AppDependencies::for_tests(
        bundle.memory_repo,
        Arc::new(InMemoryVectorStore::new()),
        bundle.embedding,
        bundle.llm,
        bundle.config,
        Arc::new(orchestrator::NoopEventPublisher),
    );
    let facade = OrchestratorFacade::new(deps);
    assert!(facade.list_memories().await.unwrap().is_empty());
}

#[tokio::test]
async fn lancedb_open_isolated() {
    let dir = tempdir().unwrap();
    let store = LancedbVectorStore::open(dir.path(), 4).await.unwrap();
    let mem = Memory::new("T", "C").unwrap();
    store
        .upsert(mem.id, &[1.0, 0.0, 0.0, 0.0])
        .await
        .unwrap();
    let emb = store.get_embedding(mem.id).await.unwrap();
    assert!(emb.is_some());
}