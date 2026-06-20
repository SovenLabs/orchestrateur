//! # Infrastructure — Adapters concrets
//!
//! Implémente les ports définis dans [`cortex::ports`].
//! Phase 3 étend ce crate (`LanceDB`, `Ollama`, `Grok`).

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod memory_repository;

pub use memory_repository::FileMemoryRepository;

/// Version du crate alignée sur le workspace.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");