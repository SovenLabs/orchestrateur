//! Tests du daemon WebSocket (feature `websocket-server`).

use orchestrator::bridge::Command;
use orchestrator::daemon::{ClientInfo, DaemonClientMessage, DaemonServerMessage, TerritoryHub};
use orchestrator::bridge::Response;
use cortex::MemoryId;

#[test]
fn daemon_protocol_roundtrip_connect() {
    let msg = DaemonClientMessage::Connect {
        token: "secret".into(),
        client: ClientInfo {
            window_kind: "main".into(),
            window_id: Some("win-1".into()),
            panels: vec!["chat".into()],
            subscriptions: vec!["chat".into()],
        },
    };
    let json = serde_json::to_string(&msg).expect("serialize");
    assert!(json.contains("\"type\":\"connect\""));
    assert!(json.contains("\"window_kind\":\"main\""));
    let back: DaemonClientMessage = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, msg);
}

#[test]
fn daemon_protocol_connect_backward_compat() {
    let json = r#"{"type":"connect","token":"dev"}"#;
    let msg: DaemonClientMessage = serde_json::from_str(json).expect("deserialize");
    match msg {
        DaemonClientMessage::Connect { token, client } => {
            assert_eq!(token, "dev");
            assert_eq!(client.window_kind, "main");
        }
        _ => panic!("expected connect"),
    }
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
fn daemon_protocol_connect_ok_with_sessions() {
    let msg = DaemonServerMessage::ConnectOk {
        version: "0.22.0".into(),
        protocol_version: shared_types::PROTOCOL_VERSION.into(),
        session_id: "sess-a".into(),
        territory_session_id: "terr-1".into(),
    };
    let json = serde_json::to_string(&msg).expect("serialize");
    assert!(json.contains("\"session_id\":\"sess-a\""));
    assert!(json.contains("\"protocol_version\":\"1.1.0\""));
    let back: DaemonServerMessage = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, msg);
}

#[test]
fn connect_ok_backward_compat_without_protocol_version() {
    let json = r#"{"type":"connect_ok","version":"0.19.0","session_id":"s","territory_session_id":"t"}"#;
    let msg: DaemonServerMessage = serde_json::from_str(json).expect("deserialize");
    match msg {
        DaemonServerMessage::ConnectOk {
            protocol_version, ..
        } => assert_eq!(protocol_version, shared_types::PROTOCOL_VERSION),
        _ => panic!("expected connect_ok"),
    }
}

#[test]
fn daemon_broadcast_roundtrip() {
    let msg = DaemonServerMessage::Broadcast {
        territory_session_id: "terr-1".into(),
        event: "memories_changed".into(),
        source_session_id: "sess-b".into(),
        payload: serde_json::json!({}),
    };
    let json = serde_json::to_string(&msg).expect("serialize");
    let back: DaemonServerMessage = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, msg);
}

#[test]
fn assimilated_response_generates_broadcast_events() {
    let events = TerritoryHub::events_from_response(
        "sess-1",
        "req-1",
        &Response::Assimilated {
            memory_id: MemoryId::new(),
            title: "Note".into(),
        },
    );
    assert!(events.iter().any(|e| e.event == "memory_assimilated"));
    assert!(events.iter().any(|e| e.event == "brain_pulse"));
}