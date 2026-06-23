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
fn unknown_broadcast_becomes_daemon_broadcast() {
    let payload = serde_json::json!({ "foo": "bar" });
    let event = BackendEvent::from_territory_broadcast("custom_event", &payload);
    match event {
        BackendEvent::DaemonBroadcast { name, .. } => assert_eq!(name, "custom_event"),
        _ => panic!("expected daemon_broadcast"),
    }
}