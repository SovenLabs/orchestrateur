//! # Cortex — Squelette de l'Orchestrateur
//!
//! Coeur technique souverain : mémoires Markdown, graphe de connaissances,
//! ports hexagonaux. Aucune dépendance vers l'infrastructure, l'IA ou l'UI.
//!
//! Garde-fous : Rust stable (`rust-toolchain.toml`), `forbid(unsafe_code)`.
//!
//! ## Architecture
//! - [`domain`] — entités, value objects, événements
//! - [`ports`] — contrats (`MemoryRepository`, `VectorStore`, `EmbeddingProvider`)
//! - [`services`] — logique métier pure (parsing Markdown, backlinks)

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod domain;
pub mod ports;
pub mod services;

pub use domain::{
    Backlink, BacklinkKind, CortexError, DomainEvent, KnowledgeGraph, Memory, MemoryAssimilated,
    MemoryId, Tag,
};
pub use ports::{EmbeddingProvider, MemoryRepository, SearchFilter, SearchHit, VectorStore};
pub use services::{
    BacklinkCalculator, BacklinkCandidate, MarkdownParser, MemoryDocument, SimilarityThresholds,
    cosine_similarity, parse_memory_markdown, serialize_memory,
};