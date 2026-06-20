//! # Orchestrateur — L'Esprit
//!
//! Phase 2 : facade, use cases, Skill Registry.
//! Ce crate ne dépend que du squelette [`cortex`] et de ses ports.
//!
//! Garde-fous : Rust stable uniquement — pas de `#![feature(...)]`.

pub mod memory_draft;

pub use memory_draft::{BacklinkDraft, BacklinkDraftKind, MemoryDraft};

/// Version du crate alignée sur le workspace.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Point d'entrée applicatif — implémenté en Phase 2.
pub struct OrchestratorApp;

impl OrchestratorApp {
    /// Crée l'application (squelette Phase 0).
    pub fn new() -> Self {
        Self
    }
}

impl Default for OrchestratorApp {
    fn default() -> Self {
        Self::new()
    }
}