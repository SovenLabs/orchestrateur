//! # Orchestrateur — L'Esprit
//!
//! Couche application : facade, use cases, Skill Registry.
//! Ne dépend que du squelette [`cortex`] et de ses ports.
//!
//! Garde-fous : Rust stable uniquement — pas de `#![feature(...)]`.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

/// Configuration applicative (`OrchestratorConfig`).
pub mod config;
/// Injection des ports Cortex (`AppDependencies`).
pub mod deps;
/// Erreurs de la couche application.
pub mod error;
/// Publication des événements de domaine.
pub mod events;
/// Facade publique stable (`OrchestratorFacade`).
pub mod facade;
/// Ports LLM — génération de [`MemoryDraft`] et chat.
pub mod llm;
/// Brouillon structuré issu des providers IA (`MemoryDraft`).
pub mod memory_draft;
/// Squelette Skills (trait, registre, noop).
pub mod skills;
/// Use cases applicatifs testables en mémoire.
pub mod use_cases;

/// Mocks in-memory des ports pour tests isolés.
pub mod testing;

pub use config::{
    ConfigError, OllamaConfig, OrchestratorConfig, ProvidersConfig, VectorStoreConfig, XaiConfig,
};
pub use deps::AppDependencies;
pub use error::{OrchestratorError, SkillError};
pub use events::{EventPublisher, NoopEventPublisher, TracingEventPublisher};
pub use facade::OrchestratorFacade;
pub use use_cases::DEFAULT_ASSIMILATION_SYSTEM_PROMPT;
pub use llm::{ChatMessage, LlmCapabilities, LlmError, LlmProvider};
pub use memory_draft::{BacklinkDraft, BacklinkDraftKind, MemoryDraft};
pub use skills::{NoopSkill, Skill, SkillContext, SkillOutput, SkillRegistry};

/// Version du crate alignée sur le workspace.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Alias historique — préférer [`OrchestratorFacade`].
pub type OrchestratorApp = OrchestratorFacade;