//! Daemon WebSocket local — couche de présentation Territoire Graphique (Option B).
//!
//! Expose le contrat bridge [`Command`] / [`Response`] en JSON sur `ws://127.0.0.1:<port>/ws`.
//! Le client Godot (ou tout autre client) se connecte au daemon Rust indépendamment du rendu.

mod error;
mod hub;
mod metrics;
mod protocol;
mod server;

pub use error::DaemonError;
pub use hub::{ConnectedWindows, TerritoryHub, WindowKind};
pub use metrics::{DaemonMetrics, DaemonMetricsSnapshot};
pub use protocol::{ClientInfo, DaemonClientMessage, DaemonServerMessage, TerritoryBroadcast};
pub use metrics::new_shared_metrics;
pub use server::{
    build_router, build_test_daemon_state, run_daemon, run_daemon_with_domain_events, serve,
    serve_with_domain_events, DaemonState, HealthResponse, MetricsResponse,
};