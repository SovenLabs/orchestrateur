//! Tests E2E Phase 1 — conversation multi-tours, persistance réelle, auto-assimilation.
//!
//! Valide la chaîne complète sans réseau : `AgentLoop` → SQLite sessions → mémoires
//! Markdown → index vectoriel → recherche sémantique.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::Path;
use std::sync::Arc;

use cortex::{SearchFilter, SessionKey, TurnRole};
use infrastructure::{FileMemoryRepository, SqliteSessionStore};
use orchestrator::agent::{AgentConfig, AgentLoop, AgentTurnRequest};
use orchestrator::testing::{
    assert_workspace_consistent, build_test_facade, InMemoryDraftRepository,
    InMemoryEmbeddingProvider, InMemoryLlmProvider, InMemoryVectorStore,
};
use orchestrator::{AppDependencies, NoopEventPublisher, OrchestratorConfig};
use tempfile::tempdir;

const TURN_MESSAGES: [&str; 3] = [
    "Tour un : l'architecture hexagonale isole le domaine Cortex des adapters infrastructure dans l'orchestrateur Rust souverain.",
    "Tour deux : le vector store indexe chaque embedding de mémoire pour permettre une recherche sémantique hybride efficace.",
    "Tour trois : SQLite persiste l'historique complet des échanges utilisateur et assistant pour chaque session agent.",
];

fn phase1_agent_config() -> AgentConfig {
    AgentConfig {
        message_preprocess: false,
        proactive_memory_search: true,
        auto_assimilate_turn: true,
        ..AgentConfig::default()
    }
}

fn phase1_config(workspace: &Path) -> OrchestratorConfig {
    let mut cfg = OrchestratorConfig::default();
    cfg.workspace_root = workspace.to_path_buf();
    cfg.embedding_dim = 8;
    cfg.vector_store.embedding_dimension = 8;
    cfg
}

async fn build_phase1_deps(workspace: &Path) -> AppDependencies {
    let cfg = phase1_config(workspace);
    std::fs::create_dir_all(cfg.memories_dir()).unwrap();
    std::fs::create_dir_all(workspace.join(".orchestrateur")).unwrap();

    let memory_repo: Arc<dyn cortex::MemoryRepository> =
        Arc::new(FileMemoryRepository::new(cfg.memories_dir()));
    let vector_store: Arc<dyn cortex::VectorStore> =
        Arc::new(InMemoryVectorStore::new());
    let embedding: Arc<dyn cortex::EmbeddingProvider> =
        Arc::new(InMemoryEmbeddingProvider::new(cfg.embedding_dim));
    let llm: Arc<dyn orchestrator::LlmProvider> = Arc::new(InMemoryLlmProvider);
    let session_repo: Arc<dyn cortex::SessionRepository> = Arc::new(
        SqliteSessionStore::open(&cfg.sessions_db_path()).expect("ouverture SQLite sessions"),
    );
    let draft_repo: Arc<dyn orchestrator::DraftRepository> =
        Arc::new(InMemoryDraftRepository::new());

    AppDependencies::for_tests(
        memory_repo,
        vector_store,
        embedding,
        llm,
        session_repo,
        draft_repo,
        cfg,
        Arc::new(NoopEventPublisher),
    )
}

fn count_memory_files(memories_dir: &Path) -> usize {
    std::fs::read_dir(memories_dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|e| {
                    e.path()
                        .extension()
                        .is_some_and(|ext| ext == "md")
                })
                .count()
        })
        .unwrap_or(0)
}

#[tokio::test]
async fn phase1_multi_turn_conversation_persists_and_searchable() {
    let dir = tempdir().unwrap();
    let deps = build_phase1_deps(dir.path()).await;
    let session_key = SessionKey::new("phase1-multi").unwrap();
    let agent = AgentLoop::new(deps.clone(), phase1_agent_config(), None);

    for message in TURN_MESSAGES {
        let result = agent
            .run_turn(AgentTurnRequest {
                session_key: session_key.clone(),
                message: message.into(),
            })
            .await
            .expect("tour agent");
        assert_eq!(result.reply, message);
        assert!(result.tools_invoked.is_empty());
    }

    let turns = deps.session_repo.list_turns(&session_key).await.unwrap();
    assert_eq!(turns.len(), 6, "3 tours = 6 entrées user+assistant");
    assert!(turns.iter().any(|t| t.role == TurnRole::User));
    assert!(turns.iter().any(|t| t.role == TurnRole::Assistant));
    assert!(
        deps.config.sessions_db_path().exists(),
        "sessions.db doit exister sur disque"
    );

    let memories_dir = deps.config.memories_dir();
    assert!(
        count_memory_files(&memories_dir) >= 1,
        "au moins une mémoire Markdown doit être créée"
    );

    assert_workspace_consistent(&deps)
        .await
        .expect("cohérence dépôt / vector store");

    let facade = build_test_facade(deps);
    let hits = facade
        .search_memories("hexagonale Cortex", &SearchFilter::default())
        .await
        .expect("recherche vectorielle");
    assert!(
        !hits.is_empty(),
        "la mémoire du premier tour doit être retrouvable par recherche sémantique"
    );
}

#[tokio::test]
async fn phase1_auto_assimilation_writes_memory_markdown() {
    let dir = tempdir().unwrap();
    let deps = build_phase1_deps(dir.path()).await;
    let memories_dir = deps.config.memories_dir();
    assert_eq!(count_memory_files(&memories_dir), 0);

    let agent = AgentLoop::new(deps.clone(), phase1_agent_config(), None);
    let message = "Décision architecturale : chaque tour agent substantiel déclenche une assimilation \
        automatique vers le Cortex, avec persistance Markdown et indexation vectorielle immédiate.";

    let result = agent
        .run_turn(AgentTurnRequest {
            session_key: SessionKey::new("phase1-auto").unwrap(),
            message: message.into(),
        })
        .await
        .expect("tour avec auto-assimilation");

    assert!(
        result.auto_assimilated.is_some(),
        "auto_assimilated doit signaler la création de mémoire"
    );
    assert_eq!(count_memory_files(&memories_dir), 1);

    let memories = deps.memory_repo.list().await.unwrap();
    assert_eq!(memories.len(), 1);
    assert!(
        memories[0].content.contains("assimilation"),
        "le contenu assimilé doit refléter l'échange"
    );

    assert_workspace_consistent(&deps)
        .await
        .expect("cohérence après auto-assimilation");
}