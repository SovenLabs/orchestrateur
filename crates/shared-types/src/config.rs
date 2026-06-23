//! Configuration client WebSocket partagée entre desktop Tauri et clients visuels.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Configuration client WebSocket (Tauri / Godot / panels).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../apps/tauri-desktop/src/lib/generated/")]
pub struct ConnectionConfig {
    /// URL WebSocket (`ws://127.0.0.1:28790/ws`).
    pub ws_url: String,
    /// Token d'authentification daemon.
    pub token: String,
    /// Intervalle heartbeat en millisecondes.
    pub heartbeat_ms: u64,
    /// Délai initial de reconnexion en millisecondes.
    pub reconnect_base_ms: u64,
    /// Délai maximal de reconnexion en millisecondes.
    pub reconnect_max_ms: u64,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            ws_url: "ws://127.0.0.1:28790/ws".into(),
            token: "dev".into(),
            heartbeat_ms: 15_000,
            reconnect_base_ms: 500,
            reconnect_max_ms: 30_000,
        }
    }
}