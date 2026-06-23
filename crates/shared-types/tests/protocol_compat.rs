use orchestrator::bridge::Command;
use orchestrator::daemon::DaemonClientMessage as TypedClient;
use shared_types::DaemonClientMessage as SharedClient;

#[test]
fn typed_connect_roundtrips_through_shared_protocol() {
    let typed = TypedClient::Connect {
        token: "dev".into(),
        protocol_version: shared_types::PROTOCOL_VERSION.into(),
        client: Default::default(),
    };
    let json = serde_json::to_string(&typed).expect("serialize typed");
    let shared: SharedClient = serde_json::from_str(&json).expect("deserialize shared");
    let back: TypedClient = TypedClient::try_from(shared).expect("convert back");
    assert_eq!(back, typed);
}

#[test]
fn typed_execute_roundtrips_through_shared_protocol() {
    let typed = TypedClient::Execute {
        request_id: "req-1".into(),
        command: Command::HealthCheck,
    };
    let json = serde_json::to_string(&typed).expect("serialize typed");
    let shared: SharedClient = serde_json::from_str(&json).expect("deserialize shared");
    let back: TypedClient = TypedClient::try_from(shared).expect("convert back");
    assert_eq!(back, typed);
}

#[test]
fn connect_backward_compat_shared() {
    let json = r#"{"type":"connect","token":"dev"}"#;
    let shared: SharedClient = serde_json::from_str(json).expect("deserialize");
    let typed: TypedClient = TypedClient::try_from(shared).expect("convert");
    match typed {
        TypedClient::Connect {
            token,
            protocol_version,
            client,
        } => {
            assert_eq!(token, "dev");
            assert_eq!(protocol_version, shared_types::PROTOCOL_VERSION);
            assert_eq!(client.window_kind, "main");
        }
        _ => panic!("expected connect"),
    }
}