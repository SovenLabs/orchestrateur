//! Événements UI et commandes frontend du protocole territorial v2.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Événement émis par le backend vers les clients (UI Tauri, Godot, panels).
///
/// Les broadcasts daemon (`brain_pulse`, `memory_assimilated`, …) sont normalisés
/// en variantes typées pour éviter le parsing ad hoc côté frontend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../apps/tauri-desktop/src/lib/generated/")]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum BackendEvent {
    /// Niveau d'activité agent (0.0–1.0).
    AgentActivity {
        /// Intensité normalisée.
        level: f32,
    },
    /// Mémoire assimilée dans le Cortex.
    MemoryAssimilated {
        /// Identifiant mémoire.
        memory_id: String,
        /// Intensité visuelle suggérée.
        intensity: f32,
    },
    /// Brouillon créé par le watcher (file de revue).
    DraftCreated {
        /// Identifiant brouillon.
        draft_id: String,
        /// Titre candidat.
        title: String,
        /// Kind sérialisé (`decision`, `context`, …).
        kind: String,
    },
    /// Brouillon publié en mémoire Cortex.
    DraftPublished {
        /// Identifiant brouillon.
        draft_id: String,
        /// Identifiant mémoire créée.
        memory_id: String,
    },
    /// Brouillon rejeté sans publication.
    DraftDiscarded {
        /// Identifiant brouillon.
        draft_id: String,
    },
    /// Propagation d'une pensée dans le graphe neuronal.
    ThoughtPropagation {
        /// Chemin de neurones stimulés.
        path: Vec<u32>,
    },
    /// Statut système global.
    SystemStatus {
        /// Libellé (`ok`, `degraded`, `error`, …).
        status: String,
    },
    /// Neurone stimulé (sphère Godot).
    NeuronStimulated {
        /// Identifiant neurone.
        id: u32,
        /// Intensité.
        intensity: f32,
    },
    /// Événement broadcast brut du daemon (fallback typé).
    DaemonBroadcast {
        /// Nom d'événement territorial.
        name: String,
        /// Charge utile JSON.
        #[ts(type = "Record<string, unknown>")]
        payload: serde_json::Value,
    },
    /// Connexion daemon établie.
    Connected {
        /// Version orchestrateur.
        version: String,
        /// Session client WS.
        session_id: String,
        /// Session territoire partagée.
        territory_session_id: String,
    },
    /// Déconnexion ou perte de lien.
    Disconnected {
        /// Raison lisible.
        reason: String,
    },
}

/// Commande émise par le frontend vers le backend / daemon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../apps/tauri-desktop/src/lib/generated/")]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum FrontendCommand {
    /// Demande un snapshot des mémoires récentes.
    RequestMemorySnapshot,
    /// Déclenche une pensée / pulse visuel.
    TriggerThought {
        /// Intensité (0.0–1.0).
        intensity: f32,
    },
    /// Keepalive explicite (complète le ping WS natif).
    Heartbeat {
        /// Nonce client.
        nonce: u64,
    },
}

impl BackendEvent {
    /// Convertit un broadcast daemon territorial en événement UI typé.
    #[must_use]
    pub fn from_territory_broadcast(event: &str, payload: &serde_json::Value) -> Self {
        match event {
            "brain_pulse" => Self::AgentActivity {
                level: payload
                    .get("level")
                    .and_then(serde_json::Value::as_f64)
                    .map_or(0.5, |v| v as f32),
            },
            "memory_assimilated" => Self::MemoryAssimilated {
                memory_id: payload
                    .get("memory_id")
                    .or_else(|| payload.get("id"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                intensity: payload
                    .get("intensity")
                    .and_then(serde_json::Value::as_f64)
                    .map_or(0.7, |v| v as f32),
            },
            "draft_created" => Self::DraftCreated {
                draft_id: payload
                    .get("draft_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                title: payload
                    .get("title")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                kind: payload
                    .get("kind")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("context")
                    .to_string(),
            },
            "draft_published" => Self::DraftPublished {
                draft_id: payload
                    .get("draft_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                memory_id: payload
                    .get("memory_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            },
            "draft_discarded" => Self::DraftDiscarded {
                draft_id: payload
                    .get("draft_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            },
            "degraded_mode" => Self::SystemStatus {
                status: "degraded".into(),
            },
            "system_error" => Self::SystemStatus {
                status: "error".into(),
            },
            "thought_propagation" => Self::ThoughtPropagation {
                path: payload
                    .get("path")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_u64().map(|n| n as u32))
                            .collect()
                    })
                    .unwrap_or_default(),
            },
            "neuron_stimulated" => Self::NeuronStimulated {
                id: payload
                    .get("id")
                    .and_then(serde_json::Value::as_u64)
                    .map_or(0, |v| v as u32),
                intensity: payload
                    .get("intensity")
                    .and_then(serde_json::Value::as_f64)
                    .map_or(0.5, |v| v as f32),
            },
            other => Self::DaemonBroadcast {
                name: other.to_string(),
                payload: payload.clone(),
            },
        }
    }
}