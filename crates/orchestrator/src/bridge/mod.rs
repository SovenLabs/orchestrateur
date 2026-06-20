//! Bridge de communication HUD ↔ Orchestrateur (Phase 4).
//!
//! Contrat découplé : la présentation envoie des [`Command`] et reçoit des [`Response`]
//! via le trait [`OrchestratorHandle`], sans accès direct aux ports Cortex.
//!
//! # Architecture
//!
//! ```text
//! HUD (egui) ──Command──► flume ──► thread Tokio ──► OrchestratorFacade
//!              ◄──Response── flume ◄──
//!              ◄──DomainEvent── FanoutEventPublisher
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

pub use command::Command;
pub use error::BridgeError;
pub use events::FanoutEventPublisher;
pub use handle::{ChannelHandle, OrchestratorHandle};
pub use response::Response;
pub use runtime::{format_assimilate_user_prompt, spawn_orchestrator_bridge, OrchestratorThread};
pub use types::{AppError, BridgeSearchHit, MemorySummary};