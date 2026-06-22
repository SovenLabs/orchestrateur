use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use cortex::DomainEvent;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::bridge::{Command, Response};

use super::protocol::{DaemonServerMessage, TerritoryBroadcast};

/// Type de fenêtre Territoire Graphique (Godot).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowKind {
    /// Fenêtre principale — seule autorisée pour actions critiques + Boule.
    Main,
    /// Extension du territoire — panneau(s) détaché(s), pas de Boule.
    Extension,
}

impl WindowKind {
    /// Parse le champ `window_kind` du handshake client.
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        if raw.eq_ignore_ascii_case("extension") {
            Self::Extension
        } else {
            Self::Main
        }
    }
}

/// Client WebSocket enregistré dans le hub territorial.
pub struct ClientSession {
    /// Identifiant unique de session (généré par le daemon).
    pub session_id: String,
    /// Type de fenêtre.
    pub window_kind: WindowKind,
    /// Topics d'événements broadcast acceptés.
    pub subscriptions: HashSet<String>,
    /// Canal sortant vers la boucle WS du client.
    pub outbound: mpsc::UnboundedSender<DaemonServerMessage>,
}

/// Hub central — fan-out broadcast vers les clients WS (source unique de vérité).
#[derive(Debug, Clone)]
pub struct TerritoryHub {
    territory_session_id: Arc<String>,
    clients: Arc<Mutex<HashMap<String, ClientSession>>>,
}

impl TerritoryHub {
    /// Crée un hub avec un identifiant de territoire stable pour la durée du daemon.
    #[must_use]
    pub fn new() -> Self {
        Self {
            territory_session_id: Arc::new(Uuid::now_v7().to_string()),
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Identifiant de session territorial partagé par toutes les fenêtres Godot.
    #[must_use]
    pub fn territory_session_id(&self) -> String {
        self.territory_session_id.as_ref().clone()
    }

    /// Enregistre un client connecté.
    pub fn register(&self, session: ClientSession) {
        if let Ok(mut guard) = self.clients.lock() {
            guard.insert(session.session_id.clone(), session);
        }
    }

    /// Retire un client déconnecté.
    pub fn unregister(&self, session_id: &str) {
        if let Ok(mut guard) = self.clients.lock() {
            guard.remove(session_id);
        }
    }

    /// Nombre de clients connectés (tests / observabilité).
    #[must_use]
    pub fn client_count(&self) -> usize {
        self.clients
            .lock()
            .map(|guard| guard.len())
            .unwrap_or(0)
    }

    /// Diffuse un événement aux clients abonnés, en excluant optionnellement l'émetteur.
    pub fn broadcast(&self, event: TerritoryBroadcast, exclude_session: Option<&str>) {
        let Ok(guard) = self.clients.lock() else {
            return;
        };
        for (id, client) in guard.iter() {
            if exclude_session == Some(id.as_str()) {
                continue;
            }
            if !client.subscriptions.contains(&event.event)
                && !client.subscriptions.contains("visual")
            {
                continue;
            }
            if Self::is_brain_only(&event.event) && client.window_kind != WindowKind::Main {
                continue;
            }
            let _ = client.outbound.send(DaemonServerMessage::Broadcast {
                territory_session_id: self.territory_session_id(),
                event: event.event.clone(),
                source_session_id: event.source_session_id.clone(),
                payload: event.payload.clone(),
            });
        }
    }

    /// Diffuse à tous les clients (événements domaine Cortex).
    pub fn broadcast_all(&self, event: TerritoryBroadcast) {
        self.broadcast(event, None);
    }

    fn is_brain_only(event: &str) -> bool {
        matches!(
            event,
            "brain_pulse"
                | "memory_assimilated"
                | "tool_call"
                | "vector_search"
                | "system_error"
        )
    }

    /// Mappe un [`DomainEvent`] Cortex vers des broadcasts territoriaux.
    #[must_use]
    pub fn events_from_domain_event(event: &DomainEvent) -> Vec<TerritoryBroadcast> {
        let source = "cortex".to_string();
        match event {
            DomainEvent::MemoryAssimilated(payload) => vec![
                TerritoryBroadcast {
                    event: "memory_assimilated".into(),
                    source_session_id: source.clone(),
                    payload: json!({
                        "memory_id": payload.memory_id.to_string(),
                        "backlink_count": payload.backlink_count,
                    }),
                },
                TerritoryBroadcast {
                    event: "memories_changed".into(),
                    source_session_id: source.clone(),
                    payload: json!({}),
                },
                TerritoryBroadcast {
                    event: "graph_changed".into(),
                    source_session_id: source.clone(),
                    payload: json!({}),
                },
                TerritoryBroadcast {
                    event: "brain_pulse".into(),
                    source_session_id: source,
                    payload: json!({"boost": 0.75, "duration": 0.85, "kind": "assimilation"}),
                },
            ],
            DomainEvent::KnowledgeGraphValidated(payload) => vec![TerritoryBroadcast {
                event: "graph_changed".into(),
                source_session_id: source,
                payload: json!({
                    "node_count": payload.node_count,
                    "edge_count": payload.edge_count,
                }),
            }],
        }
    }

    /// Dérive les broadcasts à émettre après une commande `execute`.
    #[must_use]
    pub fn events_from_response(
        source_session_id: &str,
        request_id: &str,
        response: &Response,
    ) -> Vec<TerritoryBroadcast> {
        let mut events = Vec::new();
        match response {
            Response::Assimilated { memory_id, title } => {
                events.push(TerritoryBroadcast {
                    event: "memory_assimilated".into(),
                    source_session_id: source_session_id.into(),
                    payload: json!({
                        "memory_id": memory_id.to_string(),
                        "title": title,
                    }),
                });
                events.push(TerritoryBroadcast {
                    event: "memories_changed".into(),
                    source_session_id: source_session_id.into(),
                    payload: json!({}),
                });
                events.push(TerritoryBroadcast {
                    event: "graph_changed".into(),
                    source_session_id: source_session_id.into(),
                    payload: json!({}),
                });
                events.push(TerritoryBroadcast {
                    event: "brain_pulse".into(),
                    source_session_id: source_session_id.into(),
                    payload: json!({"boost": 0.75, "duration": 0.85, "kind": "assimilation"}),
                });
            }
            Response::ChatReply {
                reply,
                tools_invoked,
                auto_assimilated,
                auto_executed_skills,
                ..
            } => {
                events.push(TerritoryBroadcast {
                    event: "chat_reply".into(),
                    source_session_id: source_session_id.into(),
                    payload: json!({
                        "reply": reply,
                        "request_id": request_id,
                        "auto_assimilated": auto_assimilated.is_some(),
                    }),
                });
                if !tools_invoked.is_empty() {
                    events.push(TerritoryBroadcast {
                        event: "tool_call".into(),
                        source_session_id: source_session_id.into(),
                        payload: json!({
                            "tools": tools_invoked,
                            "skills": auto_executed_skills,
                        }),
                    });
                    events.push(TerritoryBroadcast {
                        event: "brain_pulse".into(),
                        source_session_id: source_session_id.into(),
                        payload: json!({"boost": 0.55, "duration": 0.45, "kind": "tool_call"}),
                    });
                } else {
                    events.push(TerritoryBroadcast {
                        event: "brain_pulse".into(),
                        source_session_id: source_session_id.into(),
                        payload: json!({"boost": 0.5, "duration": 0.6, "kind": "chat"}),
                    });
                }
                if auto_assimilated.is_some() {
                    events.push(TerritoryBroadcast {
                        event: "memories_changed".into(),
                        source_session_id: source_session_id.into(),
                        payload: json!({}),
                    });
                    events.push(TerritoryBroadcast {
                        event: "graph_changed".into(),
                        source_session_id: source_session_id.into(),
                        payload: json!({}),
                    });
                }
            }
            Response::SearchResults { items, .. } => {
                events.push(TerritoryBroadcast {
                    event: "vector_search".into(),
                    source_session_id: source_session_id.into(),
                    payload: json!({
                        "hit_count": items.len(),
                        "request_id": request_id,
                    }),
                });
                events.push(TerritoryBroadcast {
                    event: "brain_pulse".into(),
                    source_session_id: source_session_id.into(),
                    payload: json!({"boost": 0.35, "duration": 0.4, "kind": "search"}),
                });
            }
            Response::SkillResult { message } => {
                events.push(TerritoryBroadcast {
                    event: "tool_call".into(),
                    source_session_id: source_session_id.into(),
                    payload: json!({
                        "tools": ["skill"],
                        "message": message,
                    }),
                });
            }
            Response::Error(err) => {
                let degraded = err.kind == "degraded";
                events.push(TerritoryBroadcast {
                    event: if degraded {
                        "degraded_mode".into()
                    } else {
                        "system_error".into()
                    },
                    source_session_id: source_session_id.into(),
                    payload: json!({
                        "kind": err.kind,
                        "message": err.message,
                    }),
                });
                if degraded {
                    events.push(TerritoryBroadcast {
                        event: "brain_pulse".into(),
                        source_session_id: source_session_id.into(),
                        payload: json!({"boost": 0.2, "duration": 0.3, "kind": "degraded"}),
                    });
                }
            }
            Response::GraphSummary { node_count, edge_count, .. } => {
                events.push(TerritoryBroadcast {
                    event: "graph_changed".into(),
                    source_session_id: source_session_id.into(),
                    payload: json!({
                        "node_count": node_count,
                        "edge_count": edge_count,
                    }),
                });
            }
            _ => {}
        }
        events
    }
}

/// Abonnements par défaut selon le type de fenêtre et les panneaux affichés.
#[must_use]
pub fn default_subscriptions(kind: WindowKind, panels: &[String]) -> HashSet<String> {
    let mut subs = HashSet::new();
    subs.insert("activity".into());

    match kind {
        WindowKind::Main => {
            subs.insert("visual".into());
            subs.insert("memories".into());
            subs.insert("graph".into());
            subs.insert("chat".into());
            subs.insert("brain_pulse".into());
            subs.insert("memory_assimilated".into());
            subs.insert("tool_call".into());
            subs.insert("vector_search".into());
            subs.insert("system_error".into());
            subs.insert("degraded_mode".into());
        }
        WindowKind::Extension => {
            for panel in panels {
                match panel.as_str() {
                    "chat" => {
                        subs.insert("chat".into());
                        subs.insert("memories".into());
                        subs.insert("visual".into());
                    }
                    "memory" | "memories" => {
                        subs.insert("memories".into());
                        subs.insert("memory_assimilated".into());
                    }
                    "graph" => {
                        subs.insert("graph".into());
                        subs.insert("memories".into());
                    }
                    "monitoring" => {
                        subs.insert("activity".into());
                        subs.insert("degraded_mode".into());
                        subs.insert("system_error".into());
                    }
                    _ => {}
                }
            }
        }
    }
    subs
}

/// Indique si la commande est réservée à la fenêtre principale.
#[must_use]
pub fn requires_main_window(cmd: &Command) -> bool {
    matches!(
        cmd,
        Command::Assimilate { .. } | Command::ExecuteSkill { .. }
    )
}

/// Fusionne abonnements explicites client et défauts déduits des panneaux.
#[must_use]
pub fn resolve_subscriptions(
    kind: WindowKind,
    panels: &[String],
    explicit: &[String],
) -> HashSet<String> {
    let mut subs = default_subscriptions(kind, panels);
    for topic in explicit {
        if !topic.is_empty() {
            subs.insert(topic.clone());
        }
    }
    subs
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex::MemoryId;

    #[test]
    fn extension_graph_subscriptions_include_memories() {
        let subs = default_subscriptions(
            WindowKind::Extension,
            &["graph".to_string()],
        );
        assert!(subs.contains("graph"));
        assert!(subs.contains("memories"));
        assert!(!subs.contains("brain_pulse"));
    }

    #[test]
    fn assimilated_emits_memory_assimilated_event() {
        let events = TerritoryHub::events_from_response(
            "sess-1",
            "req-9",
            &Response::Assimilated {
                memory_id: MemoryId::new(),
                title: "t".into(),
            },
        );
        let kinds: Vec<_> = events.iter().map(|e| e.event.as_str()).collect();
        assert!(kinds.contains(&"memory_assimilated"));
        assert!(kinds.contains(&"brain_pulse"));
    }

    #[test]
    fn chat_with_tools_emits_tool_call() {
        let events = TerritoryHub::events_from_response(
            "sess-1",
            "req-1",
            &Response::ChatReply {
                reply: "ok".into(),
                tools_invoked: vec!["search".into()],
                auto_assimilated: None,
                auto_executed_skills: vec![],
            },
        );
        assert!(events.iter().any(|e| e.event == "tool_call"));
    }

    #[test]
    fn search_emits_vector_search() {
        use crate::bridge::BridgeSearchHit;

        let events = TerritoryHub::events_from_response(
            "sess-1",
            "req-2",
            &Response::SearchResults {
                items: vec![BridgeSearchHit {
                    memory_id: MemoryId::new(),
                    score: 0.9,
                    snippet: Some("extrait".into()),
                }],
            },
        );
        assert!(events.iter().any(|e| e.event == "vector_search"));
    }

    #[test]
    fn critical_commands_require_main() {
        assert!(requires_main_window(&Command::Assimilate {
            text: "x".into(),
            tags: vec![],
        }));
        assert!(!requires_main_window(&Command::Chat {
            message: "hi".into(),
        }));
    }
}