//! Format des messages inter-agents.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Message échangé entre agents persistants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentMessage {
    /// Identifiant unique du message.
    pub id: String,
    /// Agent émetteur.
    pub from: String,
    /// Agent destinataire.
    pub to: String,
    /// Sujet court (optionnel).
    pub subject: String,
    /// Corps du message.
    pub body: String,
    /// Horodatage ISO-8601.
    pub sent_at: String,
    /// Lu par le destinataire.
    pub read: bool,
}

impl AgentMessage {
    /// Crée un nouveau message non lu.
    #[must_use]
    pub fn new(from: impl Into<String>, to: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)).to_string(),
            from: from.into(),
            to: to.into(),
            subject: String::new(),
            body: body.into(),
            sent_at: Utc::now().to_rfc3339(),
            read: false,
        }
    }

    /// Parse l'horodatage d'envoi.
    #[must_use]
    pub fn sent_at_parsed(&self) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(&self.sent_at)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    }
}