//! Daemon WebSocket local — couche de présentation Territoire Graphique (Option B).
//!
//! Expose le contrat bridge [`Command`] / [`Response`] en JSON sur `ws://127.0.0.1:<port>/ws`.
//! Le client Godot (ou tout autre client) se connecte au daemon Rust indépendamment du rendu.

mod error;
mod protocol;
mod server;

pub use error::DaemonError;
pub use protocol::{DaemonClientMessage, DaemonServerMessage};
pub use server::{build_router, run_daemon, serve, DaemonState, HealthResponse};