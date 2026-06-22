use std::env;
use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

use crate::bridge::execute_command;
use crate::config::DaemonConfig;
use crate::facade::OrchestratorFacade;
use crate::VERSION;

use super::error::DaemonError;
use super::protocol::{DaemonClientMessage, DaemonServerMessage};

/// État partagé du daemon WebSocket.
pub struct DaemonState {
    facade: Arc<OrchestratorFacade>,
    config: DaemonConfig,
}

/// Réponse HTTP santé daemon.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Statut.
    pub status: String,
    /// Version.
    pub version: String,
    /// Port configuré.
    pub port: u16,
}

/// Construit le routeur Axum du daemon.
#[must_use]
pub fn build_router(state: Arc<DaemonState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/ws", get(ws_upgrade_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Démarre le daemon WebSocket sur `bind:port`.
///
/// # Errors
///
/// Propage [`DaemonError`] si le bind ou le serveur échoue.
pub async fn serve(
    facade: Arc<OrchestratorFacade>,
    config: &DaemonConfig,
) -> Result<(), DaemonError> {
    if !config.enabled {
        return Err(DaemonError::Config(
            "daemon désactivé — activez [daemon] enabled = true".into(),
        ));
    }

    let state = Arc::new(DaemonState {
        facade,
        config: config.clone(),
    });
    let app = build_router(state);

    let addr = format!("{}:{}", config.bind, config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| DaemonError::Network(format!("écoute sur {addr}: {e}")))?;
    info!(%addr, "daemon WebSocket démarré (Territoire Graphique)");
    axum::serve(listener, app)
        .await
        .map_err(|e| DaemonError::Network(e.to_string()))?;
    Ok(())
}

async fn health_handler(State(state): State<Arc<DaemonState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        version: VERSION.into(),
        port: state.config.port,
    })
}

async fn ws_upgrade_handler(
    State(state): State<Arc<DaemonState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_socket(state, socket))
}

async fn handle_ws_socket(state: Arc<DaemonState>, socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();
    let mut authenticated = false;

    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_)) | Err(_) => break,
            Ok(_) => continue,
        };

        let parsed: DaemonClientMessage = match serde_json::from_str(&msg) {
            Ok(m) => m,
            Err(err) => {
                let _ = send_json(
                    &mut sender,
                    DaemonServerMessage::Error {
                        request_id: None,
                        message: format!("JSON invalide: {err}"),
                    },
                )
                .await;
                continue;
            }
        };

        match parsed {
            DaemonClientMessage::Connect { token } => {
                if !verify_token(&state.config, &token) {
                    let _ = send_json(
                        &mut sender,
                        DaemonServerMessage::Error {
                            request_id: None,
                            message: "token invalide ou absent".into(),
                        },
                    )
                    .await;
                    break;
                }
                authenticated = true;
                let _ = send_json(
                    &mut sender,
                    DaemonServerMessage::ConnectOk {
                        version: VERSION.into(),
                    },
                )
                .await;
            }
            DaemonClientMessage::Ping { nonce } => {
                if !authenticated {
                    let _ = send_json(
                        &mut sender,
                        DaemonServerMessage::Error {
                            request_id: None,
                            message: "non authentifié — envoyez Connect d'abord".into(),
                        },
                    )
                    .await;
                    continue;
                }
                let _ = send_json(&mut sender, DaemonServerMessage::Pong { nonce }).await;
            }
            DaemonClientMessage::Execute {
                request_id,
                command,
            } => {
                if !authenticated {
                    let _ = send_json(
                        &mut sender,
                        DaemonServerMessage::Error {
                            request_id: Some(request_id),
                            message: "non authentifié — envoyez Connect d'abord".into(),
                        },
                    )
                    .await;
                    continue;
                }
                let response = execute_command(&state.facade, command).await;
                if let crate::bridge::Response::Error(ref err) = response {
                    warn!(kind = %err.kind, "daemon execute error");
                }
                let _ = send_json(
                    &mut sender,
                    DaemonServerMessage::Result {
                        request_id,
                        response,
                    },
                )
                .await;
            }
        }
    }
}

async fn send_json(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    message: DaemonServerMessage,
) -> Result<(), ()> {
    let text = serde_json::to_string(&message).map_err(|_| ())?;
    sender.send(Message::Text(text.into())).await.map_err(|_| ())
}

fn verify_token(config: &DaemonConfig, provided: &str) -> bool {
    let expected = match env::var(&config.token_env) {
        Ok(v) if !v.is_empty() => v,
        _ => {
            warn!(
                env = %config.token_env,
                "daemon: variable token absente — connexion refusée"
            );
            return false;
        }
    };
    constant_time_eq(provided.as_bytes(), expected.as_bytes())
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in left.iter().zip(right.iter()) {
        diff |= a ^ b;
    }
    diff == 0
}

/// Point d'entrée haut niveau : démarre le daemon avec la configuration fournie.
///
/// # Errors
///
/// Propage [`DaemonError`] si le daemon ne peut pas démarrer.
pub async fn run_daemon(
    facade: Arc<OrchestratorFacade>,
    config: &DaemonConfig,
) -> Result<(), DaemonError> {
    serve(facade, config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_time_eq_matches() {
        assert!(constant_time_eq(b"secret", b"secret"));
        assert!(!constant_time_eq(b"secret", b"other"));
        assert!(!constant_time_eq(b"short", b"longer"));
    }
}