//! Providers de repli et résolution via registre typé Phase 9.

mod resolve;
mod unavailable;

pub use resolve::{resolve_embedding_from_registry, resolve_llm_from_registry};
pub use unavailable::{UnavailableEmbeddingProvider, UnavailableLlmProvider};
