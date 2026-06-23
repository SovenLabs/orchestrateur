use shared_types::BackendEvent;

#[test]
fn brain_pulse_maps_to_agent_activity() {
    let payload = serde_json::json!({ "level": 0.8 });
    let event = BackendEvent::from_territory_broadcast("brain_pulse", &payload);
    match event {
        BackendEvent::AgentActivity { level } => assert!((level - 0.8).abs() < f32::EPSILON),
        _ => panic!("expected agent_activity"),
    }
}

#[test]
fn thought_propagation_maps_correctly() {
    let payload = serde_json::json!({ "path": [1, 2, 3] });
    let event = BackendEvent::from_territory_broadcast("thought_propagation", &payload);
    match event {
        BackendEvent::ThoughtPropagation { path } => assert_eq!(path, vec![1, 2, 3]),
        _ => panic!("expected thought_propagation"),
    }
}

#[test]
fn draft_created_maps_correctly() {
    let payload = serde_json::json!({
        "draft_id": "d-1",
        "title": "Décision",
        "kind": "decision"
    });
    let event = BackendEvent::from_territory_broadcast("draft_created", &payload);
    match event {
        BackendEvent::DraftCreated {
            draft_id,
            title,
            kind,
        } => {
            assert_eq!(draft_id, "d-1");
            assert_eq!(title, "Décision");
            assert_eq!(kind, "decision");
        }
        _ => panic!("expected draft_created"),
    }
}

#[test]
fn draft_published_maps_correctly() {
    let payload = serde_json::json!({
        "draft_id": "d-1",
        "memory_id": "m-1"
    });
    let event = BackendEvent::from_territory_broadcast("draft_published", &payload);
    match event {
        BackendEvent::DraftPublished {
            draft_id,
            memory_id,
        } => {
            assert_eq!(draft_id, "d-1");
            assert_eq!(memory_id, "m-1");
        }
        _ => panic!("expected draft_published"),
    }
}

#[test]
fn draft_discarded_maps_correctly() {
    let payload = serde_json::json!({ "draft_id": "d-2" });
    let event = BackendEvent::from_territory_broadcast("draft_discarded", &payload);
    match event {
        BackendEvent::DraftDiscarded { draft_id } => assert_eq!(draft_id, "d-2"),
        _ => panic!("expected draft_discarded"),
    }
}

#[test]
fn agent_status_changed_maps_correctly() {
    let payload = serde_json::json!({
        "agent_id": "researcher",
        "status": "awake"
    });
    let event = BackendEvent::from_territory_broadcast("agent_status_changed", &payload);
    match event {
        BackendEvent::AgentStatusChanged { agent_id, status } => {
            assert_eq!(agent_id, "researcher");
            assert_eq!(status, "awake");
        }
        _ => panic!("expected agent_status_changed"),
    }
}

#[test]
fn agent_message_received_maps_correctly() {
    let payload = serde_json::json!({
        "to": "beta",
        "from": "alpha",
        "message_id": "msg-1"
    });
    let event = BackendEvent::from_territory_broadcast("agent_message", &payload);
    match event {
        BackendEvent::AgentMessageReceived {
            to,
            from,
            message_id,
        } => {
            assert_eq!(to, "beta");
            assert_eq!(from, "alpha");
            assert_eq!(message_id, "msg-1");
        }
        _ => panic!("expected agent_message"),
    }
}

#[test]
fn delegation_completed_maps_correctly() {
    let payload = serde_json::json!({
        "delegation_id": "del-9",
        "status": "done"
    });
    let event = BackendEvent::from_territory_broadcast("delegation_completed", &payload);
    match event {
        BackendEvent::DelegationCompleted {
            delegation_id,
            status,
        } => {
            assert_eq!(delegation_id, "del-9");
            assert_eq!(status, "done");
        }
        _ => panic!("expected delegation_completed"),
    }
}

#[test]
fn unknown_broadcast_becomes_daemon_broadcast() {
    let payload = serde_json::json!({ "foo": "bar" });
    let event = BackendEvent::from_territory_broadcast("custom_event", &payload);
    match event {
        BackendEvent::DaemonBroadcast { name, .. } => assert_eq!(name, "custom_event"),
        _ => panic!("expected daemon_broadcast"),
    }
}