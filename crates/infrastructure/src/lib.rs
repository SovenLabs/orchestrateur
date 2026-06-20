//! # Infrastructure — Adapters concrets
//!
//! Implémentations des ports Cortex et Orchestrator : fichiers, `LanceDB`, `Ollama`, `xAI`.
//!
//! Garde-fous : `#![forbid(unsafe_code)]`, I/O async, factories sans panique.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod embedding;
mod http_retry;
mod http_status;
mod llm;
mod memory_repository;
mod providers;
mod vector_store;
mod wiring;

pub use embedding::{
    build_embedding_provider, ChainedEmbeddingProvider, EmbeddingFactoryError,
    OllamaEmbeddingProvider,
};
pub use llm::{
    build_llm_provider, ChainedLlmProvider, LlmFactoryError, OllamaLlmProvider, XaiGrokProvider,
};
pub use memory_repository::FileMemoryRepository;
pub use vector_store::{build_vector_store, LancedbVectorStore, VectorStoreFactoryError};
pub use wiring::{build_app_dependencies, WiringError};
