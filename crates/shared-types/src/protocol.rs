//! Messages WebSocket daemon ↔ clients (handshake, broadcast, erreurs).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

fn default_true() -> bool {
    true
}

/// Capacités harness déclarées par le client au handshake.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, Default)]
#[ts(export, export_to = "../../../apps/tauri-desktop/src/lib/generated/")]
pub struct HarnessCapabilities {
    /// Peut écrire le Cortex (assimilate, publish draft, …).
    #[serde(default = "default_true")]
    pub can_write_cortex: bool,
    /// Peut exécuter des skills Esprit.
    #[serde(default = "default_true")]
    pub can_run_skills: bool,
}

/// Métadonnées fenêtre client — handshake `connect`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../apps/tauri-desktop/src/lib/generated/")]
pub struct ClientInfo {
    /// `main`, `extension`, `desktop` (Tauri), ou `sphere` (Godot standalone).
    #[serde(default = "default_window_kind")]
    pub window_kind: String,
    /// Identifiant stable côté client (optionnel).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_id: Option<String>,
    /// Panneaux affichés (`chat`, `memory`, `graph`, `monitoring`).
    #[serde(default)]
    pub panels: Vec<String>,
    /// Topics broadcast explicites (fusionnés avec les défauts).
    #[serde(default)]
    pub subscriptions: Vec<String>,
    /// Capacités harness (desktop et main : écriture par défaut).
    #[serde(default)]
    pub harness: HarnessCapabilities,
}

fn default_window_kind() -> String {
    "main".to_string()
}

impl Default for ClientInfo {
    fn default() -> Self {
        Self {
            window_kind: default_window_kind(),
            window_id: None,
            panels: Vec::new(),
            subscriptions: Vec::new(),
            harness: HarnessCapabilities::default(),
        }
    }
}

fn default_protocol_version() -> String {
    crate::PROTOCOL_VERSION.to_string()
}

/// Événement territorial diffusé à plusieurs clients WS.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../apps/tauri-desktop/src/lib/generated/")]
pub struct TerritoryBroadcast {
    /// Nom de l'événement (`memories_changed`, `brain_pulse`, …).
    pub event: String,
    /// Session source ayant déclenché l'événement.
    pub source_session_id: String,
    /// Charge utile JSON libre.
    #[ts(type = "Record<string, unknown>")]
    pub payload: Value,
}

/// Message client → daemon (WebSocket JSON).
///
/// `command` dans `Execute` est un [`Value`] pour permettre l'export TS sans dépendre
/// du crate `orchestrator`. Le daemon désérialise en [`Command`](orchestrator::bridge::Command).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../apps/tauri-desktop/src/lib/generated/")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonClientMessage {
    /// Handshake initial avec token.
    Connect {
        /// Token (`ORCHESTRATEUR_DAEMON_TOKEN`).
        token: String,
        /// Version protocole client (optionnel — défaut `1.0.0`).
        #[serde(default = "default_protocol_version")]
        protocol_version: String,
        /// Métadonnées fenêtre.
        #[serde(default)]
        client: ClientInfo,
    },
    /// Exécute une commande bridge.
    Execute {
        /// Identifiant de corrélation.
        request_id: String,
        /// Commande bridge sérialisée (`{"command":"HealthCheck"}` ou avec payload).
        #[ts(type = "Record<string, unknown>")]
        command: Value,
    },
    /// Keepalive.
    Ping {
        /// Nonce renvoyé dans `Pong`.
        nonce: u64,
    },
}

/// Message daemon → client (WebSocket JSON).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../apps/tauri-desktop/src/lib/generated/")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonServerMessage {
    /// Handshake accepté.
    ConnectOk {
        /// Version orchestrateur (crate).
        version: String,
        /// Version protocole WS ([`crate::PROTOCOL_VERSION`]).
        #[serde(default = "default_protocol_version")]
        protocol_version: String,
        /// Identifiant de session de ce client WS.
        session_id: String,
        /// Identifiant de territoire partagé.
        territory_session_id: String,
    },
    /// Réponse bridge pour une requête `Execute`.
    Result {
        /// Identifiant de corrélation.
        request_id: String,
        /// Réponse bridge sérialisée.
        #[ts(type = "Record<string, unknown>")]
        response: Value,
    },
    /// Événement broadcast territorial.
    Broadcast {
        /// Territoire partagé.
        territory_session_id: String,
        /// Nom d'événement.
        event: String,
        /// Session ayant initié l'action.
        source_session_id: String,
        /// Charge utile.
        #[ts(type = "Record<string, unknown>")]
        payload: Value,
    },
    /// Réponse keepalive.
    Pong {
        /// Nonce reçu.
        nonce: u64,
    },
    /// Erreur protocole ou exécution.
    Error {
        /// Identifiant de requête (si applicable).
        #[serde(skip_serializing_if = "Option::is_none")]
        request_id: Option<String>,
        /// Message d'erreur lisible.
        message: String,
    },
}