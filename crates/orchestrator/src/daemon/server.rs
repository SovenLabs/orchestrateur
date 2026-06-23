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
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use cortex::DomainEvent;
use shared_types::{is_client_version_supported, PROTOCOL_VERSION};

use crate::bridge::execute_command;
use crate::config::DaemonConfig;
use crate::facade::OrchestratorFacade;
use crate::VERSION;

use super::error::DaemonError;
use super::hub::{
    can_harness_write, requires_harness_write, resolve_subscriptions, ClientSession,
    ConnectedWindows, TerritoryHub,
    WindowKind,
};
use super::metrics::{new_shared_metrics, DaemonMetrics, DaemonMetricsSnapshot};
use super::protocol::{DaemonClientMessage, DaemonServerMessage};

/// État partagé du daemon WebSocket.
pub struct DaemonState {
    facade: Arc<OrchestratorFacade>,
    config: DaemonConfig,
    hub: TerritoryHub,
    metrics: Arc<DaemonMetrics>,
}

impl DaemonState {
    /// Compteurs observabilité daemon.
    #[must_use]
    pub fn metrics(&self) -> Arc<DaemonMetrics> {
        self.metrics.clone()
    }
}

/// Réponse HTTP santé daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Statut.
    pub status: String,
    /// Version.
    pub version: String,
    /// Version protocole WS.
    pub protocol_version: String,
    /// Port configuré.
    pub port: u16,
    /// Identifiant de session territorial actif.
    pub territory_session_id: String,
    /// Nombre de clients WS connectés.
    pub connected_clients: usize,
    /// Répartition par type de fenêtre (Phase 25).
    pub connected_windows: ConnectedWindows,
    /// Métriques temps réel.
    pub metrics: DaemonMetricsSnapshot,
}

/// Réponse HTTP métriques dédiées (`/metrics`).
#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    /// Version protocole.
    pub protocol_version: String,
    /// Clients connectés.
    pub connected_clients: usize,
    /// Compteurs daemon.
    pub metrics: DaemonMetricsSnapshot,
}

/// Construit un état daemon pour tests d'intégration (Phase 23).
#[must_use]
pub fn build_test_daemon_state(
    facade: Arc<OrchestratorFacade>,
    config: DaemonConfig,
) -> Arc<DaemonState> {
    let metrics = new_shared_metrics();
    let hub = TerritoryHub::with_metrics(Some(metrics.clone()));
    Arc::new(DaemonState {
        facade,
        config,
        hub,
        metrics,
    })
}

/// Construit le routeur Axum du daemon.
#[must_use]
pub fn build_router(state: Arc<DaemonState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
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

    let metrics = new_shared_metrics();
    let hub = TerritoryHub::with_metrics(Some(metrics.clone()));
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

    let hub_for_watcher = hub.clone();
    let on_draft_ready: crate::watcher::DraftReadyCallback = Arc::new(move |stored| {
        use serde_json::json;

        let summary = stored.to_summary();
        hub_for_watcher.broadcast_all(super::protocol::TerritoryBroadcast {
            event: "draft_created".into(),
            source_session_id: "watcher".into(),
            payload: json!({
                "draft_id": summary.id,
                "title": summary.title,
                "kind": summary.kind,
                "status": summary.status,
            }),
        });
    });
    crate::watcher::spawn_if_enabled(Arc::clone(&facade), Some(on_draft_ready));

    let state = Arc::new(DaemonState {
        facade,
        config: config.clone(),
        hub,
        metrics,
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
        protocol_version: PROTOCOL_VERSION.into(),
        port: state.config.port,
        territory_session_id: state.hub.territory_session_id(),
        connected_clients: state.hub.client_count(),
        connected_windows: state.hub.connected_windows(),
        metrics: state.metrics.snapshot(),
    })
}

async fn metrics_handler(State(state): State<Arc<DaemonState>>) -> Json<MetricsResponse> {
    Json(MetricsResponse {
        protocol_version: PROTOCOL_VERSION.into(),
        connected_clients: state.hub.client_count(),
        metrics: state.metrics.snapshot(),
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
                        if send_json(&mut sender, &state.metrics, message).await.is_err() {
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

                state.metrics.inc_received();

                let parsed: DaemonClientMessage = match serde_json::from_str(&msg) {
                    Ok(m) => m,
                    Err(err) => {
                        state.metrics.inc_parse_error();
                        let _ = send_json(
                            &mut sender,
                            &state.metrics,
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
                    DaemonClientMessage::Connect {
                        token,
                        protocol_version,
                        client,
                    } => {
                        if !verify_token(&state.config, &token) {
                            state.metrics.inc_auth_failure();
                            let _ = send_json(
                                &mut sender,
                                &state.metrics,
                                DaemonServerMessage::Error {
                                    request_id: None,
                                    message: "token invalide ou absent".into(),
                                },
                            )
                            .await;
                            break;
                        }
                        if !is_client_version_supported(&protocol_version) {
                            warn!(
                                client_protocol = %protocol_version,
                                min = shared_types::PROTOCOL_MIN_CLIENT,
                                "client protocole obsolète"
                            );
                        }
                        window_kind = WindowKind::parse(&client.window_kind);
                        let subscriptions = resolve_subscriptions(
                            window_kind,
                            &client.panels,
                            &client.subscriptions,
                        );
                        authenticated = true;
                        state.metrics.inc_connection();
                        state.hub.register(ClientSession {
                            session_id: session_id.clone(),
                            window_kind,
                            subscriptions,
                            outbound: out_tx.clone(),
                        });
                        debug!(
                            %session_id,
                            window_kind = ?window_kind,
                            client_protocol = %protocol_version,
                            "daemon client connecté"
                        );
                        let _ = send_json(
                            &mut sender,
                            &state.metrics,
                            DaemonServerMessage::ConnectOk {
                                version: VERSION.into(),
                                protocol_version: PROTOCOL_VERSION.into(),
                                session_id: session_id.clone(),
                                territory_session_id: state.hub.territory_session_id(),
                            },
                        )
                        .await;
                    }
                    DaemonClientMessage::Ping { nonce } => {
                        state.metrics.inc_ping();
                        if !authenticated {
                            let _ = send_json(
                                &mut sender,
                                &state.metrics,
                                DaemonServerMessage::Error {
                                    request_id: None,
                                    message: "non authentifié — envoyez Connect d'abord".into(),
                                },
                            )
                            .await;
                            continue;
                        }
                        let _ = send_json(
                            &mut sender,
                            &state.metrics,
                            DaemonServerMessage::Pong { nonce },
                        )
                        .await;
                    }
                    DaemonClientMessage::Execute {
                        request_id,
                        command,
                    } => {
                        state.metrics.inc_execute();
                        if !authenticated {
                            let _ = send_json(
                                &mut sender,
                                &state.metrics,
                                DaemonServerMessage::Error {
                                    request_id: Some(request_id),
                                    message: "non authentifié — envoyez Connect d'abord".into(),
                                },
                            )
                            .await;
                            continue;
                        }
                        if requires_harness_write(&command) && !can_harness_write(window_kind) {
                            let _ = send_json(
                                &mut sender,
                                &state.metrics,
                                DaemonServerMessage::Result {
                                    request_id: request_id.clone(),
                                    response: crate::bridge::Response::Error(
                                        crate::bridge::AppError {
                                            kind: "harness_write_denied".into(),
                                            message: format!(
                                                "écriture harness refusée pour window_kind={}",
                                                window_kind.as_str()
                                            ),
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
                            &state.metrics,
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
    metrics: &DaemonMetrics,
    message: DaemonServerMessage,
) -> Result<(), ()> {
    let text = serde_json::to_string(&message).map_err(|_| ())?;
    metrics.inc_sent();
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