//! Client Bridge mutualisé pour les peaux TUI et HUD.
//!
//! Encapsule [`spawn_orchestrator_bridge`] et factorise la logique commune
//! d'interprétation des [`Response`] / [`DomainEvent`].

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

mod client;
mod response;

pub use client::{ClientError, OrchestratorClient};
pub use orchestrator::{
    bridge::{BridgeError, Command, OrchestratorHandle, Response},
    AppDependencies, ChannelHandle, DomainEvent, OrchestratorThread,
};
pub use response::{
    domain_event_action, graph_status_message, AuditUpdate, BridgeUiAction, DomainEventAction,
    GraphUpdate, HealthUpdate,
};