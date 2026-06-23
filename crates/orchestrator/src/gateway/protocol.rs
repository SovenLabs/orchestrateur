//! Protocole JSON typé client ↔ gateway (messages, streaming).

use serde::{Deserialize, Serialize};

/// Message client → gateway (WebSocket JSON).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GatewayClientMessage {
    /// Handshake initial avec token.
    Connect {
        /// Token Bearer (variable d'environnement configurée).
        token: String,
        /// Clé de session optionnelle (défaut `default`).
        session_key: Option<String>,
    },
    /// Demande d'un tour agent.
    AgentSend {
        /// Identifiant de corrélation client.
        request_id: String,
        /// Clé de session.
        session_key: String,
        /// Message utilisateur.
        message: String,
        /// Canal source (ex. `webchat`, `telegram`).
        channel: Option<String>,
    },
    /// Keepalive.
    Ping {
        /// Nonce renvoyé dans `Pong`.
        nonce: u64,
    },
}

/// Message gateway → client (WebSocket JSON).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GatewayServerMessage {
    /// Handshake accepté.
    ConnectOk {
        /// Version orchestrateur.
        version: String,
        /// Clé de session effective.
        session_key: String,
    },
    /// Fragment de réponse assistant (`agent.stream.delta`).
    AgentStreamDelta {
        /// Identifiant de requête.
        request_id: String,
        /// Delta textuel.
        delta: String,
    },
    /// Événement outil (`agent.stream.tool`).
    AgentStreamTool {
        /// Identifiant de requête.
        request_id: String,
        /// Nom de l'outil.
        tool: String,
        /// `start` ou `end`.
        phase: String,
        /// Succès (phase `end` uniquement).
        #[serde(skip_serializing_if = "Option::is_none")]
        success: Option<bool>,
    },
    /// Fin de tour (`agent.stream.end`).
    AgentStreamEnd {
        /// Identifiant de requête.
        request_id: String,
        /// Réponse finale.
        reply: String,
        /// Outils invoqués.
        tools_invoked: Vec<String>,
    },
    /// Erreur agent ou protocole.
    AgentError {
        /// Identifiant de requête (si applicable).
        #[serde(skip_serializing_if = "Option::is_none")]
        request_id: Option<String>,
        /// Message d'erreur.
        message: String,
    },
    /// Réponse keepalive.
    Pong {
        /// Nonce reçu.
        nonce: u64,
    },
    /// Santé gateway.
    Health {
        /// Statut textuel.
        status: String,
        /// Version orchestrateur.
        version: String,
    },
}

impl GatewayServerMessage {
    /// Construit un delta de streaming.
    #[must_use]
    pub fn stream_delta(request_id: impl Into<String>, delta: impl Into<String>) -> Self {
        Self::AgentStreamDelta {
            request_id: request_id.into(),
            delta: delta.into(),
        }
    }

    /// Construit un événement outil.
    #[must_use]
    pub fn stream_tool(
        request_id: impl Into<String>,
        tool: impl Into<String>,
        phase: impl Into<String>,
        success: Option<bool>,
    ) -> Self {
        Self::AgentStreamTool {
            request_id: request_id.into(),
            tool: tool.into(),
            phase: phase.into(),
            success,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_client_connect() {
        let msg = GatewayClientMessage::Connect {
            token: "secret".into(),
            session_key: Some("chat-1".into()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: GatewayClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn roundtrip_server_stream_end() {
        let msg = GatewayServerMessage::AgentStreamEnd {
            request_id: "req-1".into(),
            reply: "Bonjour".into(),
            tools_invoked: vec!["memory_search".into()],
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("agent_stream_end"));
    }
}