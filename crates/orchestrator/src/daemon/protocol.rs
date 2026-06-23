use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::bridge::{Command, Response};
use shared_types::PROTOCOL_VERSION;

pub use shared_types::protocol::{ClientInfo, TerritoryBroadcast};

fn default_protocol_version() -> String {
    PROTOCOL_VERSION.to_string()
}

/// Message client → daemon (WebSocket JSON) — variante typée bridge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonClientMessage {
    /// Handshake initial avec token Bearer.
    Connect {
        /// Token (variable d'environnement configurée dans `orchestrator.toml`).
        token: String,
        /// Version protocole client.
        #[serde(default = "default_protocol_version")]
        protocol_version: String,
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

/// Message daemon → client (WebSocket JSON) — variante typée bridge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonServerMessage {
    /// Handshake accepté.
    ConnectOk {
        /// Version orchestrateur.
        version: String,
        /// Version protocole WS négociée.
        #[serde(default = "default_protocol_version")]
        protocol_version: String,
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

impl From<DaemonClientMessage> for shared_types::DaemonClientMessage {
    fn from(value: DaemonClientMessage) -> Self {
        match value {
            DaemonClientMessage::Connect {
                token,
                protocol_version,
                client,
            } => Self::Connect {
                token,
                protocol_version,
                client,
            },
            DaemonClientMessage::Execute {
                request_id,
                command,
            } => {
                let command = match serde_json::to_value(command) {
                    Ok(value) => value,
                    Err(_) => Value::Null,
                };
                Self::Execute {
                    request_id,
                    command,
                }
            }
            DaemonClientMessage::Ping { nonce } => Self::Ping { nonce },
        }
    }
}

impl TryFrom<shared_types::DaemonClientMessage> for DaemonClientMessage {
    type Error = serde_json::Error;

    fn try_from(value: shared_types::DaemonClientMessage) -> Result<Self, Self::Error> {
        Ok(match value {
            shared_types::DaemonClientMessage::Connect {
                token,
                protocol_version,
                client,
            } => Self::Connect {
                token,
                protocol_version,
                client,
            },
            shared_types::DaemonClientMessage::Execute {
                request_id,
                command,
            } => Self::Execute {
                request_id,
                command: serde_json::from_value(command)?,
            },
            shared_types::DaemonClientMessage::Ping { nonce } => Self::Ping { nonce },
        })
    }
}

impl From<DaemonServerMessage> for shared_types::DaemonServerMessage {
    fn from(value: DaemonServerMessage) -> Self {
        match value {
            DaemonServerMessage::ConnectOk {
                version,
                protocol_version,
                session_id,
                territory_session_id,
            } => Self::ConnectOk {
                version,
                protocol_version,
                session_id,
                territory_session_id,
            },
            DaemonServerMessage::Result {
                request_id,
                response,
            } => {
                let response = match serde_json::to_value(response) {
                    Ok(value) => value,
                    Err(_) => Value::Null,
                };
                Self::Result {
                    request_id,
                    response,
                }
            }
            DaemonServerMessage::Broadcast {
                territory_session_id,
                event,
                source_session_id,
                payload,
            } => Self::Broadcast {
                territory_session_id,
                event,
                source_session_id,
                payload,
            },
            DaemonServerMessage::Pong { nonce } => Self::Pong { nonce },
            DaemonServerMessage::Error {
                request_id,
                message,
            } => Self::Error {
                request_id,
                message,
            },
        }
    }
}