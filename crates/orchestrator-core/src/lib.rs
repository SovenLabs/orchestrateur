//! Cœur souverain Orchestrateur v2 — placeholder Phase 21.
//!
//! À terme : extraction du Cortex runtime, AgentLoop et coordination daemon
//! depuis le crate `orchestrator`. Pour l'instant, la logique métier reste
//! dans `crates/orchestrator` ; ce crate expose uniquement les contrats partagés.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

use shared_types::BackendEvent;
use tracing::info;

/// Version sémantique du module core (alignée workspace).
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Point d'entrée placeholder — journalise le démarrage du core v2.
pub fn init() {
    info!(version = CORE_VERSION, "orchestrator-core initialisé (placeholder Phase 21)");
}

/// Convertit un événement territorial en événement UI typé.
#[must_use]
pub fn map_broadcast(event: &str, payload: &serde_json::Value) -> BackendEvent {
    BackendEvent::from_territory_broadcast(event, payload)
}