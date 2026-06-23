use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// Compteurs atomiques du daemon WebSocket (observabilité Phase 23).
#[derive(Debug, Default)]
pub struct DaemonMetrics {
    messages_received: AtomicU64,
    messages_sent: AtomicU64,
    broadcasts_sent: AtomicU64,
    execute_requests: AtomicU64,
    ping_requests: AtomicU64,
    connections_opened: AtomicU64,
    auth_failures: AtomicU64,
    parse_errors: AtomicU64,
}

impl DaemonMetrics {
    /// Incrémente les messages entrants.
    pub fn inc_received(&self) {
        self.messages_received.fetch_add(1, Ordering::Relaxed);
    }

    /// Incrémente les messages sortants.
    pub fn inc_sent(&self) {
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
    }

    /// Incrémente les broadcasts territoriaux.
    pub fn inc_broadcast(&self) {
        self.broadcasts_sent.fetch_add(1, Ordering::Relaxed);
    }

    /// Incrémente les requêtes `execute`.
    pub fn inc_execute(&self) {
        self.execute_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Incrémente les pings.
    pub fn inc_ping(&self) {
        self.ping_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Incrémente les connexions authentifiées.
    pub fn inc_connection(&self) {
        self.connections_opened.fetch_add(1, Ordering::Relaxed);
    }

    /// Incrémente les échecs d'authentification.
    pub fn inc_auth_failure(&self) {
        self.auth_failures.fetch_add(1, Ordering::Relaxed);
    }

    /// Incrémente les erreurs de parsing JSON.
    pub fn inc_parse_error(&self) {
        self.parse_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Capture un instantané sérialisable pour `/health`.
    #[must_use]
    pub fn snapshot(&self) -> DaemonMetricsSnapshot {
        DaemonMetricsSnapshot {
            messages_received: self.messages_received.load(Ordering::Relaxed),
            messages_sent: self.messages_sent.load(Ordering::Relaxed),
            broadcasts_sent: self.broadcasts_sent.load(Ordering::Relaxed),
            execute_requests: self.execute_requests.load(Ordering::Relaxed),
            ping_requests: self.ping_requests.load(Ordering::Relaxed),
            connections_opened: self.connections_opened.load(Ordering::Relaxed),
            auth_failures: self.auth_failures.load(Ordering::Relaxed),
            parse_errors: self.parse_errors.load(Ordering::Relaxed),
        }
    }
}

/// Instantané des métriques daemon (JSON `/health`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonMetricsSnapshot {
    /// Messages JSON reçus.
    pub messages_received: u64,
    /// Messages JSON envoyés.
    pub messages_sent: u64,
    /// Broadcasts territoriaux émis.
    pub broadcasts_sent: u64,
    /// Requêtes `execute` traitées.
    pub execute_requests: u64,
    /// Pings traités.
    pub ping_requests: u64,
    /// Connexions authentifiées cumulées.
    pub connections_opened: u64,
    /// Échecs d'authentification.
    pub auth_failures: u64,
    /// Erreurs de parsing JSON.
    pub parse_errors: u64,
}

/// Crée un [`Arc`] partagé pour le daemon.
#[must_use]
pub fn new_shared_metrics() -> Arc<DaemonMetrics> {
    Arc::new(DaemonMetrics::default())
}