use serde::{Deserialize, Serialize};

use crate::bridge::{Command, Response};

/// Message client → daemon (WebSocket JSON).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonClientMessage {
    /// Handshake initial avec token Bearer.
    Connect {
        /// Token (variable d'environnement configurée dans `orchestrator.toml`).
        token: String,
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonServerMessage {
    /// Handshake accepté.
    ConnectOk {
        /// Version orchestrateur.
        version: String,
    },
    /// Réponse bridge pour une requête `Execute`.
    Result {
        /// Identifiant de corrélation.
        request_id: String,
        /// Réponse bridge.
        response: Response,
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