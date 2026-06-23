//! Tests du dépôt fichier de brouillons (infrastructure).

use std::sync::Arc;

use cortex::MemoryDraft;
use infrastructure::FileDraftRepository;
use orchestrator::draft::{DraftRepository, DraftStatus};
use orchestrator::testing::InMemorySessionRepository;
use orchestrator::{AppDependencies, OrchestratorConfig, OrchestratorFacade};
use tempfile::tempdir;

#[tokio::test]
async fn file_draft_repo_wired_via_facade() {
    let dir = tempdir().unwrap();
    let mut config = OrchestratorConfig::default();
    config.workspace_root = dir.path().to_path_buf();

    let draft_repo: Arc<dyn DraftRepository> =
        Arc::new(FileDraftRepository::new(config.drafts_dir()));
    let deps = AppDependencies::for_tests(
        Arc::new(orchestrator::testing::InMemoryMemoryRepository::new()),
        Arc::new(orchestrator::testing::InMemoryVectorStore::new()),
        Arc::new(orchestrator::testing::InMemoryEmbeddingProvider::new(
            config.embedding_dim,
        )),
        Arc::new(orchestrator::testing::InMemoryLlmProvider),
        Arc::new(InMemorySessionRepository::new()),
        draft_repo,
        config,
        Arc::new(orchestrator::NoopEventPublisher),
    );

    let facade = OrchestratorFacade::new(deps);
    let stored = facade
        .store_draft(MemoryDraft::new("Fichier", "corps"), None)
        .await
        .unwrap();

    let path = dir
        .path()
        .join(".orchestrateur/drafts")
        .join(format!("{}.json", stored.id));
    assert!(path.exists());

    let (draft_id, (memory, _)) = facade.publish_draft(&stored.id).await.unwrap();
    assert_eq!(draft_id, stored.id);
    assert_eq!(memory.title, "Fichier");

    let reloaded = facade.get_draft(&stored.id).await.unwrap();
    assert_eq!(reloaded.status, DraftStatus::Published);
}