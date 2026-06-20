//! Réexport de l'interprétation mutualisée (source : `orchestrator::bridge::ui_response`).

pub use orchestrator::{
    domain_event_action, graph_status_message, AuditUpdate, BridgeUiAction, GraphUpdate,
    HealthUpdate,
};

/// Alias sémantique pour les événements domaine.
pub type DomainEventAction = BridgeUiAction;