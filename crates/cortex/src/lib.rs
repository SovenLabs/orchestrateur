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

/// Entités, value objects et événements du domaine Cortex.
pub mod domain;
/// Contrats hexagonaux (traits async des ports).
pub mod ports;
/// Services de domaine purs (parsing Markdown, backlinks).
pub mod services;

pub use domain::{
    Backlink, BacklinkDraft, BacklinkDraftKind, BacklinkKind, ConversationTurn, CortexError,
    DomainEvent, KnowledgeGraph, KnowledgeGraphValidated, Memory, MemoryAssimilated, MemoryDraft,
    MemoryId, Session, SessionKey, Tag, TurnRole,
};
pub use ports::{
    Embedding, EmbeddingCapabilities, EmbeddingError, EmbeddingProvider, MemoryRepository,
    SearchFilter, SearchHit, SessionRepository, VectorStore,
};
pub use services::{
    cosine_similarity, find_injection_pattern, parse_memory_markdown, serialize_memory,
    BacklinkCalculator, BacklinkCandidate, MarkdownParser, MemoryDocument, MemoryDraftValidator,
    MemoryDraftValidatorConfig, SimilarityThresholds, ValidationError, ValidationResult,
    ValidationWarning,
};
