//! Mocks in-memory des ports Cortex pour tests isolés (sans disque ni réseau).

mod mocks;

pub use mocks::{
    InMemoryEmbeddingProvider, InMemoryLlmProvider, InMemoryMemoryRepository,
    InMemoryVectorStore, MockBundle,
};