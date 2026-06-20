//! # Orchestrateur — L'Esprit
//!
//! Couche application : facade, use cases, Skill Registry.
//! Ne dépend que du squelette [`cortex`] et de ses ports.
//!
//! Garde-fous : Rust stable uniquement — pas de `#![feature(...)]`.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

/// Contrat HUD ↔ orchestrateur (`Command`, `Response`, `OrchestratorHandle`).
pub mod bridge;
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
/// Sondes de disponibilité providers (health bridge).
pub mod health;
/// Ports LLM — génération de [`MemoryDraft`] et chat.
pub mod llm;
/// Brouillon structuré issu des providers IA (`MemoryDraft`).
pub mod memory_draft;
/// Défense en profondeur — validation adversariale des sorties LLM.
pub mod security;
/// Squelette Skills (trait, registre, noop).
pub mod skills;
/// Interface terminal ratatui (feature `tui` uniquement).
#[cfg(feature = "tui")]
pub mod tui;
/// Use cases applicatifs testables en mémoire.
pub mod use_cases;

/// Mocks in-memory des ports pour tests isolés.
pub mod testing;

pub use bridge::{
    format_assimilate_user_prompt, spawn_orchestrator_bridge, AppError, BridgeError,
    BridgeSearchHit, ChannelHandle, Command, FanoutEventPublisher, MemorySummary,
    OrchestratorHandle, OrchestratorThread, Response,
};
pub use config::{
    AuditConfig, BehavioralConfig, ConfigError, IntegrityConfig, OllamaConfig, OrchestratorConfig,
    ProvidersConfig, SecurityConfig, VectorStoreConfig, XaiConfig,
};
pub use cortex::DomainEvent;
pub use deps::AppDependencies;
pub use error::{OrchestratorError, SkillError};
pub use events::{EventPublisher, NoopEventPublisher, TracingEventPublisher};
pub use facade::OrchestratorFacade;
pub use llm::{ChatMessage, LlmCapabilities, LlmError, LlmProvider, LlmUsageRecorded};
pub use memory_draft::{BacklinkDraft, BacklinkDraftKind, MemoryDraft};
pub use security::{
    build_security_context, build_test_security_context, BehavioralError, IntegrityStatus,
    MemoryDraftValidator, SecurityBootstrapError, SecurityContext, SecurityGateError,
    SecurityProfile, ValidationError,
};
pub use skills::{NoopSkill, Skill, SkillContext, SkillOutput, SkillRegistry};
#[cfg(feature = "tui")]
pub use tui::TuiApp;
pub use use_cases::DEFAULT_ASSIMILATION_SYSTEM_PROMPT;

/// Version du crate alignée sur le workspace.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Alias historique — préférer [`OrchestratorFacade`].
pub type OrchestratorApp = OrchestratorFacade;
