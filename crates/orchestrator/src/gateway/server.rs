use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::{header::HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use cortex::SessionKey;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

use crate::VERSION;

use super::channels::{WebhookChannel, WebhookPayload};
use super::error::GatewayError;
use super::{resolve_channel_config, verify_channel_token};
use super::protocol::{GatewayClientMessage, GatewayServerMessage};
use super::registry::InboundMessage;
use super::runtime::GatewayRunner;

/// État partagé du serveur gateway.
pub struct GatewayState {
    runner: Arc<GatewayRunner>,
    webhook: Arc<WebhookChannel>,
}

/// Réponse HTTP santé gateway.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Statut.
    pub status: String,
    /// Version.
    pub version: String,
    /// Port configuré.
    pub port: u16,
}

/// Réponse webhook synchrone.
#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    /// Réponse agent.
    pub reply: String,
    /// Clé de session utilisée.
    pub session_key: String,
}

/// Construit le routeur Axum du gateway.
#[must_use]
pub fn build_router(state: Arc<GatewayState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/ws", get(ws_upgrade_handler))
        .route("/v1/channels/webhook", post(webhook_handler))
        .route(
            "/v1/channels/{channel_id}/inbound",
            post(channel_inbound_handler),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Démarre le serveur gateway sur `bind:port`.
///
/// # Errors
///
/// Propage [`GatewayError`] si le bind ou le serveur échoue.
pub async fn serve(
    runner: Arc<GatewayRunner>,
    webhook: Arc<WebhookChannel>,
) -> Result<(), GatewayError> {
    let config = runner.gateway_config().clone();
    let state = Arc::new(GatewayState { runner, webhook });
    let app = build_router(state);

    let addr = format!("{}:{}", config.bind, config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| GatewayError::Network(format!("écoute sur {addr}: {e}")))?;
    info!(%addr, "gateway WebSocket démarré");
    axum::serve(listener, app)
        .await
        .map_err(|e| GatewayError::Network(e.to_string()))?;
    Ok(())
}

async fn health_handler(State(state): State<Arc<GatewayState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        version: VERSION.into(),
        port: state.runner.gateway_config().port,
    })
}

async fn ws_upgrade_handler(
    State(state): State<Arc<GatewayState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_socket(state, socket))
}

async fn channel_inbound_handler(
    State(state): State<Arc<GatewayState>>,
    Path(channel_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<WebhookPayload>,
) -> Result<Json<WebhookResponse>, (StatusCode, String)> {
    if state.runner.registry().get(&channel_id).is_none() {
        return Err((StatusCode::NOT_FOUND, format!("canal inconnu: {channel_id}")));
    }
    let gateway_cfg = state.runner.gateway_config();
    let channel_cfg = resolve_channel_config(gateway_cfg, &channel_id);
    let token = headers
        .get("x-orchestrateur-channel-token")
        .and_then(|v| v.to_str().ok());
    verify_channel_token(&channel_id, &channel_cfg, token)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "token canal invalide".into()))?;

    let inbound = InboundMessage {
        channel_id: channel_id.clone(),
        session_key: payload.session_key.clone(),
        text: payload.message.clone(),
        external_id: payload.external_id.clone(),
    };

    let handler = state.runner.inbound_handler();
    let reply = handler
        .handle(inbound)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(WebhookResponse {
        reply,
        session_key: payload.session_key,
    }))
}

async fn webhook_handler(
    State(state): State<Arc<GatewayState>>,
    headers: HeaderMap,
    Json(payload): Json<WebhookPayload>,
) -> Result<Json<WebhookResponse>, (StatusCode, String)> {
    let secret = headers
        .get("x-orchestrateur-webhook-secret")
        .and_then(|v| v.to_str().ok());
    state
        .webhook
        .verify_secret(secret)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "secret webhook invalide".into()))?;

    let inbound = InboundMessage {
        channel_id: "webhook".into(),
        session_key: payload.session_key.clone(),
        text: payload.message.clone(),
        external_id: payload.external_id.clone(),
    };

    let handler = state.runner.inbound_handler();
    let reply = handler
        .handle(inbound)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(WebhookResponse {
        reply,
        session_key: payload.session_key,
    }))
}

async fn handle_ws_socket(state: Arc<GatewayState>, socket: WebSocket) {
    let (mut sink, mut stream) = socket.split();
    let mut connected = false;
    let mut session_key = SessionKey::default_chat();

    while let Some(msg) = stream.next().await {
        let msg = match msg {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_)) => break,
            Ok(_) => continue,
            Err(err) => {
                warn!(%err, "websocket lecture échouée");
                break;
            }
        };

        let client_msg: GatewayClientMessage = match serde_json::from_str(&msg) {
            Ok(parsed) => parsed,
            Err(err) => {
                let _ = send_json(
                    &mut sink,
                    GatewayServerMessage::AgentError {
                        request_id: None,
                        message: format!("JSON invalide: {err}"),
                    },
                )
                .await;
                continue;
            }
        };

        match client_msg {
            GatewayClientMessage::Connect { token, session_key: sk } => {
                if state.runner.verify_token(&token).is_err() {
                    let _ = send_json(
                        &mut sink,
                        GatewayServerMessage::AgentError {
                            request_id: None,
                            message: "token invalide".into(),
                        },
                    )
                    .await;
                    break;
                }
                if let Some(key) = sk {
                    match SessionKey::new(key) {
                        Ok(parsed) => session_key = parsed,
                        Err(err) => {
                            let _ = send_json(
                                &mut sink,
                                GatewayServerMessage::AgentError {
                                    request_id: None,
                                    message: err.to_string(),
                                },
                            )
                            .await;
                            continue;
                        }
                    }
                }
                connected = true;
                let _ = send_json(
                    &mut sink,
                    GatewayServerMessage::ConnectOk {
                        version: VERSION.into(),
                        session_key: session_key.to_string(),
                    },
                )
                .await;
            }
            GatewayClientMessage::Ping { nonce } => {
                let _ = send_json(&mut sink, GatewayServerMessage::Pong { nonce }).await;
            }
            GatewayClientMessage::AgentSend {
                request_id,
                session_key: sk,
                message,
                channel,
            } => {
                if !connected {
                    let _ = send_json(
                        &mut sink,
                        GatewayServerMessage::AgentError {
                            request_id: Some(request_id.clone()),
                            message: "connect requis avant agent.send".into(),
                        },
                    )
                    .await;
                    continue;
                }
                let parsed_key = match SessionKey::new(sk) {
                    Ok(key) => key,
                    Err(err) => {
                        let _ = send_json(
                            &mut sink,
                            GatewayServerMessage::AgentError {
                                request_id: Some(request_id.clone()),
                                message: err.to_string(),
                            },
                        )
                        .await;
                        continue;
                    }
                };
                let channel_id = channel.unwrap_or_else(|| "webchat".into());
                let (stream_tx, stream_rx) = flume::unbounded::<GatewayServerMessage>();
                let runner = Arc::clone(&state.runner);
                let req_id = request_id.clone();
                let turn_handle = tokio::spawn(async move {
                    runner
                        .run_agent_turn(
                            &req_id,
                            parsed_key,
                            &message,
                            &channel_id,
                            Some(stream_tx),
                        )
                        .await
                });

                loop {
                    match stream_rx.recv_async().await {
                        Ok(event) => {
                            if send_json(&mut sink, event).await.is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }

                match turn_handle.await {
                    Ok(Ok(_)) => {}
                    Ok(Err(err)) => {
                        let _ = send_json(
                            &mut sink,
                            GatewayServerMessage::AgentError {
                                request_id: Some(request_id),
                                message: err.to_string(),
                            },
                        )
                        .await;
                    }
                    Err(err) => {
                        let _ = send_json(
                            &mut sink,
                            GatewayServerMessage::AgentError {
                                request_id: Some(request_id),
                                message: format!("tour agent interrompu: {err}"),
                            },
                        )
                        .await;
                    }
                }
            }
        }
    }
}

async fn send_json(
    sink: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    message: GatewayServerMessage,
) -> Result<(), ()> {
    let json = serde_json::to_string(&message).map_err(|_| ())?;
    sink.send(Message::Text(json.into())).await.map_err(|_| ())
}