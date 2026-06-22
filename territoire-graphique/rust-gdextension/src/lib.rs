//! GDExtension Rust — Territoire Graphique (Phase 15+).
//!
//! Ce crate sera branché sur godot-rust pour la Boule de Pixels et les shaders.
//! En Phase 14 bis, il expose uniquement les types de protocole partagés.

#![forbid(unsafe_code)]

/// Version alignée sur le workspace Orchestrateur.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// URL WebSocket par défaut du daemon local.
pub const DEFAULT_DAEMON_WS_URL: &str = "ws://127.0.0.1:28790/ws";

/// Enveloppe JSON client → daemon (miroir de `DaemonClientMessage`).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientEnvelope {
    /// Handshake avec token.
    Connect {
        /// Token Bearer.
        token: String,
    },
    /// Commande bridge sérialisée (JSON opaque en Phase 14 bis).
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
}