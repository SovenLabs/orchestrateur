//! Types partagés entre le backend Rust, le desktop Tauri et les clients visuels.
//!
//! Source de vérité pour le protocole WebSocket territorial et les événements UI v2.
//! Les types TypeScript sont générés via `cargo run -p shared-types --bin export-ts`.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

pub mod config;
pub mod events;
pub mod protocol;
pub mod protocol_version;

pub use config::ConnectionConfig;
pub use protocol_version::{is_client_version_supported, PROTOCOL_MIN_CLIENT, PROTOCOL_VERSION};
pub use events::{BackendEvent, FrontendCommand};
pub use protocol::{
    ClientInfo, DaemonClientMessage, DaemonServerMessage, HarnessCapabilities,
    TerritoryBroadcast,
};