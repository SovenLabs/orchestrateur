//! GDExtension Rust — Territoire Graphique (Phase 16).
//!
//! WebSocket : `WebSocketServer::start_websocket_server()` + client daemon.
//! Le daemon principal reste `orchestrateur daemon run` (port 28790).

#![forbid(unsafe_code)]

mod activity;
mod websocket;

pub use activity::map_health_to_activity;
pub use websocket::{WebSocketServer, WsError};

/// Version alignée sur le workspace Orchestrateur.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// URL WebSocket par défaut du daemon local.
pub const DEFAULT_DAEMON_WS_URL: &str = "ws://127.0.0.1:28790/ws";

/// Démarre le runtime WebSocket (alias public Phase 16).
///
/// # Errors
///
/// Propage [`WsError`] si le runtime tokio ne démarre pas.
pub fn start_websocket_server() -> Result<WebSocketServer, WsError> {
    WebSocketServer::start_websocket_server()
}

/// Enveloppe JSON client → daemon (miroir de `DaemonClientMessage`).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientEnvelope {
    /// Handshake avec token.
    Connect {
        /// Token Bearer.
        token: String,
    },
    /// Commande bridge sérialisée.
    Execute {
        /// Identifiant de corrélation.
        request_id: String,
        /// Corps JSON de la commande bridge.
        command: serde_json::Value,
    },
    /// Keepalive.
    Ping {
        /// Nonce.
        nonce: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ws_url_is_localhost() {
        assert!(DEFAULT_DAEMON_WS_URL.contains("28790"));
    }

    #[test]
    fn start_websocket_server_export_works() {
        assert!(start_websocket_server().is_ok());
    }
}