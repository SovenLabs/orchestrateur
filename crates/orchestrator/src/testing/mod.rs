//! Mocks in-memory des ports Cortex pour tests isolés (sans disque ni réseau).

mod event_collector;
mod hardcore_helpers;
mod mocks;
mod scripted_providers;

pub use event_collector::CollectingEventPublisher;
pub use hardcore_helpers::{
    assert_no_ghost_nodes, assert_workspace_consistent, build_test_facade, count_domain_events,
    percentile_ms, test_memories, test_memory,
};
pub use mocks::{
    InMemoryEmbeddingProvider, InMemoryLlmProvider, InMemoryMemoryRepository, InMemoryVectorStore,
    MockBundle,
};
pub use scripted_providers::{
    CountingEmbeddingProvider, FailNthVectorStore, InvalidJsonLlmProvider, ScriptedLlmProvider,
    StableOllamaLlmProvider,
};
