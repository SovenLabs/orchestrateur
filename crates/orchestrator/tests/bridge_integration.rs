//! Tests d'intégration du bridge client ↔ orchestrateur.

use std::time::Duration;

use cortex::{DomainEvent, Memory};
use orchestrator::bridge::{Command, OrchestratorHandle, Response};
use orchestrator::testing::MockBundle;
use orchestrator::{spawn_orchestrator_bridge, OrchestratorFacade};

fn wait_for_response(handle: &impl OrchestratorHandle, timeout: Duration) -> Option<Response> {
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        if let Ok(Some(response)) = handle.try_recv_response() {
            return Some(response);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    None
}

#[test]
fn bridge_chat_uses_agent_loop() {
    let deps = MockBundle::new().into_deps();
    let (handle, thread) = spawn_orchestrator_bridge(deps).unwrap();

    handle
        .send_command(Command::Chat {
            message: "Phase 7 agent".into(),
        })
        .unwrap();
    let response = wait_for_response(&handle, Duration::from_secs(2));
    drop(handle);
    thread.join();

    match response.expect("timeout chat agent") {
        Response::ChatReply { reply, .. } => assert!(reply.contains("Phase 7 agent"), "reply: {reply}"),
        other => panic!("réponse inattendue: {other:?}"),
    }
}

#[test]
#[ignore = "intégration: bridge health check (thread spawn)"]
fn bridge_health_check_roundtrip() {
    let deps = MockBundle::new().into_deps();
    let (handle, thread) = spawn_orchestrator_bridge(deps).unwrap();

    handle.send_command(Command::HealthCheck).unwrap();
    let response = wait_for_response(&handle, Duration::from_secs(2));
    drop(handle);
    thread.join();

    let response = response.expect("timeout health check");
    match response {
        Response::Health {
            status,
            version,
            llm_available,
            embedding_available,
        } => {
            assert_eq!(status, "ok");
            assert!(!version.is_empty());
            assert!(llm_available);
            assert!(embedding_available);
        }
        other => panic!("réponse inattendue: {other:?}"),
    }
}

#[test]
#[ignore = "intégration: bridge list + get memory"]
fn bridge_list_and_get_memory_roundtrip() {
    let bundle = MockBundle::new();
    let deps = bundle.into_deps();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let facade = OrchestratorFacade::new(deps.clone());
    let memory = Memory::new("Bridge Test", "Contenu bridge.").unwrap();
    let id = memory.id;
    rt.block_on(async {
        facade.save_memory(&memory).await.unwrap();
    });

    let (handle, thread) = spawn_orchestrator_bridge(deps).unwrap();
    let _events = handle.subscribe_events();

    handle
        .send_command(Command::List {
            filter: None,
            offset: 0,
            limit: 50,
        })
        .unwrap();

    let list_response = wait_for_response(&handle, Duration::from_secs(2)).expect("timeout list");
    match list_response {
        Response::MemoryList { items, total } => {
            assert_eq!(total, 1);
            assert_eq!(items[0].title, "Bridge Test");
        }
        other => panic!("réponse inattendue: {other:?}"),
    }

    handle
        .send_command(Command::GetMemory { id: id.to_string() })
        .unwrap();

    let detail_response = wait_for_response(&handle, Duration::from_secs(2)).expect("timeout get");
    match detail_response {
        Response::MemoryDetail { memory: detail } => {
            assert_eq!(detail.id, id);
            assert_eq!(detail.title, "Bridge Test");
        }
        other => panic!("réponse inattendue: {other:?}"),
    }

    drop(handle);
    thread.join();
}

#[test]
#[ignore = "intégration: bridge assimilation + événements domaine"]
fn bridge_publishes_domain_events_on_assimilate() {
    let deps = MockBundle::new().into_deps();
    let (handle, thread) = spawn_orchestrator_bridge(deps).unwrap();
    let event_rx = handle.subscribe_events();

    handle
        .send_command(Command::Assimilate {
            text: "Nouveau souvenir via bridge.".into(),
            tags: vec!["test".into()],
        })
        .unwrap();

    let _response = wait_for_response(&handle, Duration::from_secs(5));
    let event = event_rx.recv_timeout(Duration::from_secs(5)).ok();
    drop(handle);
    thread.join();

    assert!(
        event.is_some_and(|e| matches!(e, DomainEvent::MemoryAssimilated(_))),
        "événement MemoryAssimilated attendu"
    );
}

#[test]
fn bridge_command_response_are_serializable() {
    let cmd = Command::Search {
        query: "rust".into(),
        limit: 10,
    };
    let json = serde_json::to_string(&cmd).unwrap();
    let decoded: Command = serde_json::from_str(&json).unwrap();
    assert_eq!(cmd, decoded);

    let resp = Response::Success {
        message: "ok".into(),
    };
    let json = serde_json::to_string(&resp).unwrap();
    let decoded: Response = serde_json::from_str(&json).unwrap();
    assert_eq!(resp, decoded);

    let graph_cmd = Command::Graph;
    let graph_json = serde_json::to_string(&graph_cmd).unwrap();
    assert!(graph_json.contains("\"command\":\"Graph\""));

    let skill_cmd = Command::ExecuteSkill {
        name: "noop".into(),
        context: orchestrator::BridgeSkillContext::default(),
    };
    let skill_json = serde_json::to_string(&skill_cmd).unwrap();
    assert!(skill_json.contains("\"command\":\"ExecuteSkill\""));
}

#[test]
#[ignore = "intégration: bridge skills list + execute"]
fn bridge_list_skills_and_execute_noop() {
    let deps = MockBundle::new().into_deps();
    let (handle, thread) = spawn_orchestrator_bridge(deps).unwrap();

    handle.send_command(Command::ListSkills).unwrap();
    let list_response = wait_for_response(&handle, Duration::from_secs(2)).expect("timeout skills");
    match list_response {
        Response::SkillList { skills } => {
            assert!(skills.iter().any(|s| s.name == "noop"));
            assert!(skills.iter().any(|s| s.name == "list_memories"));
        }
        other => panic!("réponse inattendue: {other:?}"),
    }

    handle
        .send_command(Command::ExecuteSkill {
            name: "noop".into(),
            context: orchestrator::BridgeSkillContext::default(),
        })
        .unwrap();
    let run_response = wait_for_response(&handle, Duration::from_secs(2)).expect("timeout run");
    match run_response {
        Response::SkillResult { message } => assert_eq!(message, "noop ok"),
        other => panic!("réponse inattendue: {other:?}"),
    }

    drop(handle);
    thread.join();
}

#[test]
#[ignore = "intégration: bridge chat roundtrip"]
fn bridge_chat_roundtrip() {
    let deps = MockBundle::new().into_deps();
    let (handle, thread) = spawn_orchestrator_bridge(deps).unwrap();

    handle
        .send_command(Command::Chat {
            message: "Bonjour orchestrateur".into(),
        })
        .unwrap();
    let response = wait_for_response(&handle, Duration::from_secs(2)).expect("timeout chat");
    drop(handle);
    thread.join();

    match response {
        Response::ChatReply { reply, .. } => assert!(!reply.is_empty()),
        other => panic!("réponse inattendue: {other:?}"),
    }
}
