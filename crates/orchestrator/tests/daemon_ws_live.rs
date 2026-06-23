//! Test d'intégration live WebSocket daemon (Phase 23).

#![cfg(feature = "websocket-server")]

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use orchestrator::bridge::Command;
use orchestrator::config::DaemonConfig;
use orchestrator::daemon::{
    build_router, build_test_daemon_state, DaemonClientMessage, DaemonServerMessage, HealthResponse,
};
use orchestrator::testing::MockBundle;
use orchestrator::OrchestratorFacade;
use shared_types::PROTOCOL_VERSION;
use tokio::net::TcpListener;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

async fn spawn_test_daemon() -> (u16, Arc<orchestrator::daemon::DaemonMetrics>) {
    std::env::set_var("ORCHESTRATEUR_DAEMON_TOKEN", "phase23-test");
    let state = build_test_daemon_state(
        Arc::new(OrchestratorFacade::new(MockBundle::new().into_deps())),
        DaemonConfig {
            enabled: true,
            bind: "127.0.0.1".into(),
            port: 0,
            token_env: "ORCHESTRATEUR_DAEMON_TOKEN".into(),
        },
    );
    let metrics = state.metrics();

    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let app = build_router(state);

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    (port, metrics)
}

async fn recv_json(
    stream: &mut futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    >,
) -> DaemonServerMessage {
    for _ in 0..20 {
        if let Some(Ok(Message::Text(text))) = stream.next().await {
            if let Ok(msg) = serde_json::from_str::<DaemonServerMessage>(&text) {
                return msg;
            }
        }
    }
    panic!("aucun message JSON reçu");
}

#[tokio::test]
async fn live_connect_ping_execute_health() {
    let (port, metrics) = spawn_test_daemon().await;
    let url = format!("ws://127.0.0.1:{port}/ws");
    let (ws, _) = connect_async(&url).await.expect("connect ws");
    let (mut write, mut read) = ws.split();

    let connect = DaemonClientMessage::Connect {
        token: "phase23-test".into(),
        protocol_version: PROTOCOL_VERSION.into(),
        client: Default::default(),
    };
    write
        .send(Message::Text(
            serde_json::to_string(&connect).expect("json").into(),
        ))
        .await
        .expect("send connect");

    match recv_json(&mut read).await {
        DaemonServerMessage::ConnectOk {
            protocol_version, ..
        } => assert_eq!(protocol_version, PROTOCOL_VERSION),
        other => panic!("expected connect_ok, got {other:?}"),
    }

    write
        .send(Message::Text(
            serde_json::to_string(&DaemonClientMessage::Ping { nonce: 99 })
                .expect("json")
                .into(),
        ))
        .await
        .expect("send ping");

    match recv_json(&mut read).await {
        DaemonServerMessage::Pong { nonce } => assert_eq!(nonce, 99),
        other => panic!("expected pong, got {other:?}"),
    }

    write
        .send(Message::Text(
            serde_json::to_string(&DaemonClientMessage::Execute {
                request_id: "live-1".into(),
                command: Command::HealthCheck,
            })
            .expect("json")
            .into(),
        ))
        .await
        .expect("send execute");

    match recv_json(&mut read).await {
        DaemonServerMessage::Result { request_id, response } => {
            assert_eq!(request_id, "live-1");
            assert!(matches!(
                response,
                orchestrator::bridge::Response::Health { .. }
            ));
        }
        other => panic!("expected result, got {other:?}"),
    }

    let snapshot = metrics.snapshot();
    assert!(snapshot.messages_received >= 3);
    assert!(snapshot.messages_sent >= 3);
    assert!(snapshot.ping_requests >= 1);
    assert!(snapshot.execute_requests >= 1);
}

#[tokio::test]
async fn health_endpoint_exposes_metrics() {
    let (port, metrics) = spawn_test_daemon().await;
    metrics.inc_received();
    metrics.inc_sent();

    let url = format!("http://127.0.0.1:{port}/health");
    let client = reqwest::Client::new();
    let resp: HealthResponse = client
        .get(&url)
        .send()
        .await
        .expect("http")
        .json()
        .await
        .expect("json");

    assert_eq!(resp.protocol_version, PROTOCOL_VERSION);
    assert!(resp.metrics.messages_received >= 1);
    assert_eq!(resp.connected_windows.total, 0);
}

type WsStream = futures_util::stream::SplitStream<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
>;
type WsSink = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    Message,
>;

async fn connect_as(kind: &str, port: u16) -> (WsSink, WsStream) {
    use orchestrator::daemon::ClientInfo;

    let url = format!("ws://127.0.0.1:{port}/ws");
    let (ws, _) = connect_async(&url).await.expect("connect ws");
    let (mut write, mut read) = ws.split();
    let connect = DaemonClientMessage::Connect {
        token: "phase23-test".into(),
        protocol_version: PROTOCOL_VERSION.into(),
        client: ClientInfo {
            window_kind: kind.into(),
            ..Default::default()
        },
    };
    write
        .send(Message::Text(
            serde_json::to_string(&connect).expect("json").into(),
        ))
        .await
        .expect("send connect");
    match recv_json(&mut read).await {
        DaemonServerMessage::ConnectOk { .. } => {}
        other => panic!("expected connect_ok, got {other:?}"),
    }
    (write, read)
}

#[tokio::test]
async fn multi_window_kinds_reported_in_health() {
    let (port, _) = spawn_test_daemon().await;

    let (_desktop_write, _desktop_read) = connect_as("desktop", port).await;
    let (_sphere_write, _sphere_read) = connect_as("sphere", port).await;

    let url = format!("http://127.0.0.1:{port}/health");
    let client = reqwest::Client::new();
    let resp: HealthResponse = client
        .get(&url)
        .send()
        .await
        .expect("http")
        .json()
        .await
        .expect("json");

    assert_eq!(resp.connected_clients, 2);
    assert_eq!(resp.connected_windows.desktop, 1);
    assert_eq!(resp.connected_windows.sphere, 1);
    assert_eq!(resp.connected_windows.total, 2);
}