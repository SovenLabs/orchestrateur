//! Tests du daemon WebSocket (feature `websocket-server`).

use orchestrator::daemon::{DaemonClientMessage, DaemonServerMessage};
use orchestrator::bridge::Command;

#[test]
fn daemon_protocol_roundtrip_connect() {
    let msg = DaemonClientMessage::Connect {
        token: "secret".into(),
    };
    let json = serde_json::to_string(&msg).expect("serialize");
    assert!(json.contains("\"type\":\"connect\""));
    let back: DaemonClientMessage = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, msg);
}

#[test]
fn daemon_protocol_execute_health_check() {
    let msg = DaemonClientMessage::Execute {
        request_id: "req-1".into(),
        command: Command::HealthCheck,
    };
    let json = serde_json::to_string(&msg).expect("serialize");
    assert!(json.contains("\"command\":\"HealthCheck\""));
    let back: DaemonClientMessage = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, msg);
}

#[test]
fn daemon_protocol_connect_ok() {
    let msg = DaemonServerMessage::ConnectOk {
        version: "0.15.0".into(),
    };
    let json = serde_json::to_string(&msg).expect("serialize");
    let back: DaemonServerMessage = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, msg);
}