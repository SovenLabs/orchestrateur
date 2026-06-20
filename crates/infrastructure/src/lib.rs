//! # Infrastructure — Adapters concrets
//!
//! Phase 3 : `FileMemoryRepository`, `LancedbVectorStore`, `OllamaEmbeddingProvider`.
//! Implémente les ports définis dans [`cortex::ports`].

#![allow(dead_code)]

/// Version du crate alignée sur le workspace.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Marqueur — les adapters seront ajoutés en Phase 3.
pub struct InfrastructureLayer;