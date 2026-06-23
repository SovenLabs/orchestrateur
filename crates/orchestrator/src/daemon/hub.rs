use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use cortex::DomainEvent;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::bridge::{Command, Response};

use super::metrics::DaemonMetrics;
use super::protocol::{DaemonServerMessage, TerritoryBroadcast};

/// Type de fenêtre Territoire Graphique (Godot) ou desktop Tauri.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowKind {
    /// Fenêtre principale Godot — seule autorisée pour actions critiques + Boule intégrée.
    Main,
    /// Extension du territoire — panneau(s) détaché(s), pas de Boule.
    Extension,
    /// Interface Tauri (commandement J.A.R.V.I.S.).
    Desktop,
    /// Fenêtre Godot dédiée — Boule de Pixels Vivante standalone (Phase 25).
    Sphere,
}

impl WindowKind {
    /// Parse le champ `window_kind` du handshake client.
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        match raw.to_ascii_lowercase().as_str() {
            "extension" => Self::Extension,
            "desktop" => Self::Desktop,
            "sphere" => Self::Sphere,
            _ => Self::Main,
        }
    }

    /// Libellé wire protocol (`connect.client.window_kind`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::Extension => "extension",
            Self::Desktop => "desktop",
            Self::Sphere => "sphere",
        }
    }
}

/// Répartition des clients WS connectés par type de fenêtre.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectedWindows {
    /// Fenêtre principale Godot.
    pub main: usize,
    /// Extensions Godot détachées.
    pub extension: usize,
    /// Desktop Tauri.
    pub desktop: usize,
    /// Sphère Godot standalone.
    pub sphere: usize,
    /// Total clients authentifiés.
    pub total: usize,
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
#[derive(Clone)]
pub struct TerritoryHub {
    territory_session_id: Arc<String>,
    clients: Arc<Mutex<HashMap<String, ClientSession>>>,
    metrics: Option<Arc<DaemonMetrics>>,
}

impl TerritoryHub {
    /// Crée un hub avec un identifiant de territoire stable pour la durée du daemon.
    #[must_use]
    pub fn new() -> Self {
        Self::with_metrics(None)
    }

    /// Crée un hub avec métriques daemon optionnelles.
    #[must_use]
    pub fn with_metrics(metrics: Option<Arc<DaemonMetrics>>) -> Self {
        Self {
            territory_session_id: Arc::new(Uuid::now_v7().to_string()),
            clients: Arc::new(Mutex::new(HashMap::new())),
            metrics,
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

    /// Compte les clients connectés par `window_kind` (multi-fenêtrage Phase 25).
    #[must_use]
    pub fn connected_windows(&self) -> ConnectedWindows {
        let Ok(guard) = self.clients.lock() else {
            return ConnectedWindows {
                main: 0,
                extension: 0,
                desktop: 0,
                sphere: 0,
                total: 0,
            };
        };
        let mut counts = ConnectedWindows {
            main: 0,
            extension: 0,
            desktop: 0,
            sphere: 0,
            total: guard.len(),
        };
        for client in guard.values() {
            match client.window_kind {
                WindowKind::Main => counts.main += 1,
                WindowKind::Extension => counts.extension += 1,
                WindowKind::Desktop => counts.desktop += 1,
                WindowKind::Sphere => counts.sphere += 1,
            }
        }
        counts
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
            if client
                .outbound
                .send(DaemonServerMessage::Broadcast {
                    territory_session_id: self.territory_session_id(),
                    event: event.event.clone(),
                    source_session_id: event.source_session_id.clone(),
                    payload: event.payload.clone(),
                })
                .is_ok()
            {
                if let Some(metrics) = &self.metrics {
                    metrics.inc_sent();
                }
            }
        }
        if let Some(metrics) = &self.metrics {
            metrics.inc_broadcast();
        }
    }

    /// Diffuse à tous les clients (événements domaine Cortex).
    pub fn broadcast_all(&self, event: TerritoryBroadcast) {
        self.broadcast(event, None);
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
            Response::DraftPublished {
                draft_id,
                memory_id,
                title,
            } => {
                events.push(TerritoryBroadcast {
                    event: "draft_published".into(),
                    source_session_id: source_session_id.into(),
                    payload: json!({
                        "draft_id": draft_id,
                        "memory_id": memory_id.to_string(),
                        "title": title,
                    }),
                });
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
                    payload: json!({"boost": 0.75, "duration": 0.85, "kind": "draft_publish"}),
                });
            }
            Response::DraftDiscarded { id } => {
                events.push(TerritoryBroadcast {
                    event: "draft_discarded".into(),
                    source_session_id: source_session_id.into(),
                    payload: json!({
                        "draft_id": id,
                    }),
                });
            }
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
            Response::AgentDetail { agent } => {
                events.push(TerritoryBroadcast {
                    event: "agent_status_changed".into(),
                    source_session_id: source_session_id.into(),
                    payload: json!({
                        "agent_id": agent.id,
                        "status": agent.status,
                    }),
                });
            }
            Response::AgentTurnReply {
                reply,
                tools_invoked,
                auto_assimilated,
                ..
            } => {
                events.push(TerritoryBroadcast {
                    event: "chat_reply".into(),
                    source_session_id: source_session_id.into(),
                    payload: json!({
                        "reply": reply,
                        "request_id": request_id,
                        "persistent_agent": true,
                    }),
                });
                if !tools_invoked.is_empty() {
                    events.push(TerritoryBroadcast {
                        event: "tool_call".into(),
                        source_session_id: source_session_id.into(),
                        payload: json!({ "tools": tools_invoked }),
                    });
                }
                if auto_assimilated.is_some() {
                    events.push(TerritoryBroadcast {
                        event: "memories_changed".into(),
                        source_session_id: source_session_id.into(),
                        payload: json!({}),
                    });
                }
            }
            Response::AgentMessageSent {
                message_id,
                from,
                to,
            } => {
                events.push(TerritoryBroadcast {
                    event: "agent_message".into(),
                    source_session_id: source_session_id.into(),
                    payload: json!({
                        "message_id": message_id,
                        "from": from,
                        "to": to,
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
        WindowKind::Main | WindowKind::Sphere => {
            subs.insert("visual".into());
            subs.insert("memories".into());
            subs.insert("graph".into());
            subs.insert("chat".into());
            subs.insert("brain_pulse".into());
            subs.insert("memory_assimilated".into());
            subs.insert("tool_call".into());
            subs.insert("vector_search".into());
            subs.insert("thought_propagation".into());
            subs.insert("neuron_stimulated".into());
            subs.insert("system_error".into());
            subs.insert("degraded_mode".into());
        }
        WindowKind::Desktop => {
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

/// Indique si la commande modifie le Cortex ou exécute l'Esprit (écriture harness).
#[must_use]
pub fn requires_harness_write(cmd: &Command) -> bool {
    matches!(
        cmd,
        Command::Assimilate { .. }
            | Command::ExecuteSkill { .. }
            | Command::PublishDraft { .. }
            | Command::DiscardDraft { .. }
            | Command::WatcherStart
            | Command::WatcherStop
    )
}

/// Fenêtres autorisées pour les écritures harness (desktop Tauri + territoire principal).
#[must_use]
pub fn can_harness_write(kind: WindowKind) -> bool {
    matches!(kind, WindowKind::Main | WindowKind::Desktop)
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
    fn desktop_parses_and_subscribes_brain_pulse() {
        assert_eq!(WindowKind::parse("desktop"), WindowKind::Desktop);
        let subs = default_subscriptions(WindowKind::Desktop, &["dashboard".to_string()]);
        assert!(subs.contains("brain_pulse"));
        assert!(subs.contains("memories"));
    }

    #[test]
    fn sphere_matches_main_visual_subscriptions() {
        assert_eq!(WindowKind::parse("sphere"), WindowKind::Sphere);
        let subs = default_subscriptions(WindowKind::Sphere, &["sphere".to_string()]);
        assert!(subs.contains("brain_pulse"));
        assert!(subs.contains("thought_propagation"));
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
    fn harness_write_commands_detected() {
        assert!(requires_harness_write(&Command::Assimilate {
            text: "x".into(),
            tags: vec![],
        }));
        assert!(requires_harness_write(&Command::PublishDraft {
            id: "d1".into(),
        }));
        assert!(!requires_harness_write(&Command::Chat {
            message: "hi".into(),
        }));
    }

    #[test]
    fn desktop_can_harness_write() {
        assert!(can_harness_write(WindowKind::Desktop));
        assert!(can_harness_write(WindowKind::Main));
        assert!(!can_harness_write(WindowKind::Extension));
        assert!(!can_harness_write(WindowKind::Sphere));
    }
}