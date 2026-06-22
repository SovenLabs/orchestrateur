use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::bridge::{Command, Response};

/// Métadonnées fenêtre Godot — handshake `connect`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ClientInfo {
    /// `main` ou `extension`.
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
}

fn default_window_kind() -> String {
    "main".to_string()
}

/// Événement territorial diffusé à plusieurs clients WS.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TerritoryBroadcast {
    /// Nom de l'événement (`memories_changed`, `graph_changed`, `brain_pulse`, `chat_reply`, …).
    pub event: String,
    /// Session source ayant déclenché l'événement.
    pub source_session_id: String,
    /// Charge utile JSON libre.
    pub payload: Value,
}

/// Message client → daemon (WebSocket JSON).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonClientMessage {
    /// Handshake initial avec token Bearer.
    Connect {
        /// Token (variable d'environnement configurée dans `orchestrator.toml`).
        token: String,
        /// Métadonnées fenêtre Territoire Graphique (Phase 18).
        #[serde(default)]
        client: ClientInfo,
    },
    /// Exécute une commande bridge (`Command` sérialisé).
    Execute {
        /// Identifiant de corrélation client (renvoyé dans la réponse).
        request_id: String,
        /// Commande bridge.
        command: Command,
    },
    /// Keepalive.
    Ping {
        /// Nonce renvoyé dans `Pong`.
        nonce: u64,
    },
}

/// Message daemon → client (WebSocket JSON).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonServerMessage {
    /// Handshake accepté.
    ConnectOk {
        /// Version orchestrateur.
        version: String,
        /// Identifiant de session de ce client WS.
        session_id: String,
        /// Identifiant de territoire partagé (toutes les fenêtres Godot).
        territory_session_id: String,
    },
    /// Réponse bridge pour une requête `Execute`.
    Result {
        /// Identifiant de corrélation.
        request_id: String,
        /// Réponse bridge.
        response: Response,
    },
    /// Événement broadcast territorial (Phase 18).
    Broadcast {
        /// Territoire partagé.
        territory_session_id: String,
        /// Nom d'événement.
        event: String,
        /// Session ayant initié l'action.
        source_session_id: String,
        /// Charge utile.
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