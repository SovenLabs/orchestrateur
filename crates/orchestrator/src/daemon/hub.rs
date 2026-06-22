use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

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
            if !client.subscriptions.contains(&event.event) {
                continue;
            }
            if event.event == "brain_pulse" && client.window_kind != WindowKind::Main {
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

    /// Dérive les broadcasts à émettre après une commande `execute`.
    #[must_use]
    pub fn events_from_response(
        source_session_id: &str,
        request_id: &str,
        response: &Response,
    ) -> Vec<TerritoryBroadcast> {
        let mut events = Vec::new();
        match response {
            Response::Assimilated { .. } => {
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
                    payload: json!({"boost": 0.45, "duration": 0.5}),
                });
            }
            Response::ChatReply {
                reply,
                auto_assimilated,
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
                events.push(TerritoryBroadcast {
                    event: "brain_pulse".into(),
                    source_session_id: source_session_id.into(),
                    payload: json!({"boost": 0.5, "duration": 0.6}),
                });
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
            subs.insert("memories".into());
            subs.insert("graph".into());
            subs.insert("chat".into());
            subs.insert("brain_pulse".into());
        }
        WindowKind::Extension => {
            for panel in panels {
                match panel.as_str() {
                    "chat" => {
                        subs.insert("chat".into());
                        subs.insert("memories".into());
                    }
                    "memory" | "memories" => {
                        subs.insert("memories".into());
                    }
                    "graph" => {
                        subs.insert("graph".into());
                        subs.insert("memories".into());
                    }
                    "monitoring" => {
                        subs.insert("activity".into());
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
    fn assimilated_emits_memories_and_graph_events() {
        use cortex::MemoryId;

        let events = TerritoryHub::events_from_response(
            "sess-1",
            "req-9",
            &Response::Assimilated {
                memory_id: MemoryId::new(),
                title: "t".into(),
            },
        );
        let kinds: Vec<_> = events.iter().map(|e| e.event.as_str()).collect();
        assert!(kinds.contains(&"memories_changed"));
        assert!(kinds.contains(&"graph_changed"));
        assert!(kinds.contains(&"brain_pulse"));
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