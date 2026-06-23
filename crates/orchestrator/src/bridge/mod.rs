//! Bridge de communication client visuel ↔ Orchestrateur (daemon WS, CLI).
//!
//! Contrat découplé : la présentation envoie des [`Command`] et reçoit des [`Response`]
//! via le trait [`OrchestratorHandle`] ou le daemon WebSocket, sans accès direct aux ports Cortex.
//!
//! # Architecture
//!
//! ```text
//! Client (Godot) ──JSON/WS──► daemon ──► OrchestratorFacade
//! CLI headless   ──execute_command──► OrchestratorFacade
//! ```
//!
//! # Exemple
//!
//! ```no_run
//! use orchestrator::bridge::{spawn_orchestrator_bridge, Command, OrchestratorHandle};
//! use orchestrator::AppDependencies;
//!
//! # fn example(deps: AppDependencies) {
//! let (handle, thread) = spawn_orchestrator_bridge(deps).unwrap();
//! handle.send_command(Command::HealthCheck).unwrap();
//! thread.join();
//! # }
//! ```

mod command;
mod error;
mod events;
mod handle;
mod response;
mod runtime;
mod types;
pub mod ui_common;
pub mod ui_response;

pub use command::Command;
pub use error::BridgeError;
pub use events::FanoutEventPublisher;
pub use handle::{ChannelHandle, OrchestratorHandle};
pub use response::Response;
pub use runtime::{
    execute_command, format_assimilate_user_prompt, spawn_orchestrator_bridge, OrchestratorThread,
};
pub use ui_common::{format_health_status, MemoryDetailView};
pub use ui_response::{
    audit_from_response, domain_event_action, graph_from_response, graph_status_message,
    health_from_response, AuditUpdate, BridgeUiAction, GraphUpdate, HealthUpdate,
};
pub use types::{
    AppError, BridgeSearchHit, BridgeSkillContext, DraftSummary, HubIntegritySummary, HubSummary,
    MarketplaceEntrySummary, MemorySummary, SkillSummary, WatcherStatus,
};
