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
use tokio::sync::mpsc;
use tracing::{info, warn};
use uuid::Uuid;

use cortex::DomainEvent;

use crate::bridge::execute_command;
use crate::config::DaemonConfig;
use crate::facade::OrchestratorFacade;
use crate::VERSION;

use super::error::DaemonError;
use super::hub::{
    requires_main_window, resolve_subscriptions, ClientSession, TerritoryHub, WindowKind,
};
use super::protocol::{DaemonClientMessage, DaemonServerMessage};

/// État partagé du daemon WebSocket.
pub struct DaemonState {
    facade: Arc<OrchestratorFacade>,
    config: DaemonConfig,
    hub: TerritoryHub,
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
    /// Identifiant de session territorial actif.
    pub territory_session_id: String,
    /// Nombre de clients WS connectés.
    pub connected_clients: usize,
}

/// Construit le routeur Axum du daemon.
#[must_use]
pub fn build_router(state: Arc<DaemonState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/ws", get(ws_upgrade_handler))
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
    serve_with_domain_events(facade, config, None).await
}

/// Démarre le daemon avec abonnement optionnel aux [`DomainEvent`] Cortex (Phase 19).
///
/// # Errors
///
/// Propage [`DaemonError`] si le bind ou le serveur échoue.
pub async fn serve_with_domain_events(
    facade: Arc<OrchestratorFacade>,
    config: &DaemonConfig,
    domain_events: Option<flume::Receiver<DomainEvent>>,
) -> Result<(), DaemonError> {
    if !config.enabled {
        return Err(DaemonError::Config(
            "daemon désactivé — activez [daemon] enabled = true".into(),
        ));
    }

    let hub = TerritoryHub::new();
    if let Some(rx) = domain_events {
        let hub_clone = hub.clone();
        tokio::spawn(async move {
            while let Ok(event) = rx.recv_async().await {
                for broadcast in TerritoryHub::events_from_domain_event(&event) {
                    hub_clone.broadcast_all(broadcast);
                }
            }
        });
    }

    let state = Arc::new(DaemonState {
        facade,
        config: config.clone(),
        hub,
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
        territory_session_id: state.hub.territory_session_id(),
        connected_clients: state.hub.client_count(),
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
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<DaemonServerMessage>();
    let session_id = Uuid::now_v7().to_string();
    let mut authenticated = false;
    let mut window_kind = WindowKind::Main;

    loop {
        tokio::select! {
            outbound = out_rx.recv() => {
                match outbound {
                    Some(message) => {
                        if send_json(&mut sender, message).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
            incoming = receiver.next() => {
                let msg = match incoming {
                    Some(Ok(Message::Text(text))) => text,
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    Some(Ok(_)) => continue,
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
                    DaemonClientMessage::Connect { token, client } => {
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
                        window_kind = WindowKind::parse(&client.window_kind);
                        let subscriptions = resolve_subscriptions(
                            window_kind,
                            &client.panels,
                            &client.subscriptions,
                        );
                        authenticated = true;
                        state.hub.register(ClientSession {
                            session_id: session_id.clone(),
                            window_kind,
                            subscriptions,
                            outbound: out_tx.clone(),
                        });
                        let _ = send_json(
                            &mut sender,
                            DaemonServerMessage::ConnectOk {
                                version: VERSION.into(),
                                session_id: session_id.clone(),
                                territory_session_id: state.hub.territory_session_id(),
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
                        if requires_main_window(&command) && window_kind != WindowKind::Main {
                            let _ = send_json(
                                &mut sender,
                                DaemonServerMessage::Result {
                                    request_id: request_id.clone(),
                                    response: crate::bridge::Response::Error(
                                        crate::bridge::types::AppError {
                                            kind: "main_window_required".into(),
                                            message: "cette commande est réservée à la fenêtre principale".into(),
                                        },
                                    ),
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
                                request_id: request_id.clone(),
                                response: response.clone(),
                            },
                        )
                        .await;

                        for event in TerritoryHub::events_from_response(
                            &session_id,
                            &request_id,
                            &response,
                        ) {
                            state.hub.broadcast(event, Some(&session_id));
                        }
                    }
                }
            }
        }
    }

    state.hub.unregister(&session_id);
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
    run_daemon_with_domain_events(facade, config, None).await
}

/// Point d'entrée daemon avec fan-out des événements domaine.
///
/// # Errors
///
/// Propage [`DaemonError`] si le daemon ne peut pas démarrer.
pub async fn run_daemon_with_domain_events(
    facade: Arc<OrchestratorFacade>,
    config: &DaemonConfig,
    domain_events: Option<flume::Receiver<DomainEvent>>,
) -> Result<(), DaemonError> {
    serve_with_domain_events(facade, config, domain_events).await
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