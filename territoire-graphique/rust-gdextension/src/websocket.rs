//! Fondation WebSocket Phase 16 — runtime tokio + client daemon (Option B).
//!
//! Phase 17 : communication bidirectionnelle complète depuis GDExtension.

use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use thiserror::Error;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::activity::map_health_to_activity;
use crate::ClientEnvelope;

/// Erreurs WebSocket GDExtension.
#[derive(Debug, Error)]
pub enum WsError {
    /// Runtime tokio.
    #[error("runtime: {0}")]
    Runtime(String),
    /// Connexion ou protocole.
    #[error("websocket: {0}")]
    WebSocket(String),
    /// JSON.
    #[error("json: {0}")]
    Json(String),
}

/// Message serveur daemon (miroir `DaemonServerMessage`).
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMessage {
    ConnectOk { version: String },
    Result { request_id: String, response: serde_json::Value },
    Error { message: String },
}

/// Handle du runtime WebSocket — fondation pour GDExtension / Phase 17.
pub struct WebSocketServer {
    runtime: Runtime,
    activity: Arc<Mutex<f32>>,
}

impl WebSocketServer {
    /// Démarre le runtime async (point d'entrée `start_websocket_server`).
    ///
    /// # Errors
    ///
    /// Retourne [`WsError::Runtime`] si tokio ne démarre pas.
    pub fn start_websocket_server() -> Result<Self, WsError> {
        let runtime = Runtime::new().map_err(|e| WsError::Runtime(e.to_string()))?;
        Ok(Self {
            runtime,
            activity: Arc::new(Mutex::new(0.35)),
        })
    }

    /// Dernière intensité d'activité connue (lecture thread-safe).
    pub async fn last_activity(&self) -> f32 {
        *self.activity.lock().await
    }

    /// Lance la boucle client vers le daemon Orchestrateur (non bloquant).
    ///
    /// # Errors
    ///
    /// Retourne [`WsError::Runtime`] si le spawn échoue.
    pub fn spawn_daemon_client(&self, url: impl Into<String>, token: impl Into<String>) -> Result<(), WsError> {
        let url = url.into();
        let token = token.into();
        let activity = Arc::clone(&self.activity);
        self.runtime
            .spawn(async move {
                if let Err(err) = daemon_client_loop(&url, &token, activity).await {
                    eprintln!("[territoire-gdextension] daemon client: {err}");
                }
            });
        Ok(())
    }
}

async fn daemon_client_loop(
    url: &str,
    token: &str,
    activity: Arc<Mutex<f32>>,
) -> Result<(), WsError> {
    let (ws, _) = connect_async(url)
        .await
        .map_err(|e| WsError::WebSocket(e.to_string()))?;
    let (mut write, mut read) = ws.split();

    let connect = ClientEnvelope::Connect {
        token: token.to_string(),
    };
    let text = serde_json::to_string(&connect).map_err(|e| WsError::Json(e.to_string()))?;
    write
        .send(Message::Text(text.into()))
        .await
        .map_err(|e| WsError::WebSocket(e.to_string()))?;

    while let Some(msg) = read.next().await {
        let msg = msg.map_err(|e| WsError::WebSocket(e.to_string()))?;
        if let Message::Text(text) = msg {
            if let Some(intensity) = parse_activity_from_text(&text) {
                *activity.lock().await = intensity;
            }
            if text.contains("\"type\":\"connect_ok\"") {
                send_health_check(&mut write).await?;
            }
        }
    }
    Ok(())
}

async fn send_health_check(
    write: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        Message,
    >,
) -> Result<(), WsError> {
    let exec = serde_json::json!({
        "type": "execute",
        "request_id": "gdext-health",
        "command": { "command": "HealthCheck" }
    });
    write
        .send(Message::Text(exec.to_string().into()))
        .await
        .map_err(|e| WsError::WebSocket(e.to_string()))
}

fn parse_activity_from_text(text: &str) -> Option<f32> {
    let msg: ServerMessage = serde_json::from_str(text).ok()?;
    match msg {
        ServerMessage::Result { response, .. } => {
            if response.get("response")?.as_str()? != "Health" {
                return None;
            }
            let payload = response.get("payload")?;
            let status = payload.get("status")?.as_str()?;
            let llm = payload.get("llm_available")?.as_bool()?;
            let emb = payload.get("embedding_available")?.as_bool()?;
            Some(map_health_to_activity(status, llm, emb))
        }
        ServerMessage::ConnectOk { .. } => None,
        ServerMessage::Error { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_websocket_server_creates_runtime() {
        let server = WebSocketServer::start_websocket_server();
        assert!(server.is_ok());
    }
}