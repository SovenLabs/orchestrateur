//! Enregistrement persisté d'un brouillon en attente de revue.

use chrono::{DateTime, Utc};
use cortex::MemoryDraft;
use serde::{Deserialize, Serialize};

use crate::bridge::DraftSummary;

/// Statut du cycle de vie d'un brouillon persisté.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DraftStatus {
    /// En attente de publication ou rejet.
    Pending,
    /// Assimilé en mémoire Cortex.
    Published,
    /// Rejeté sans publication.
    Discarded,
}

impl DraftStatus {
    /// Libellé wire JSON (`pending`, `published`, `discarded`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Published => "published",
            Self::Discarded => "discarded",
        }
    }
}

fn default_pending_status() -> DraftStatus {
    DraftStatus::Pending
}

/// Brouillon persisté — métadonnées + contenu [`MemoryDraft`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredDraft {
    /// Identifiant stable (UUID).
    pub id: String,
    /// Horodatage de création UTC.
    pub created_at: DateTime<Utc>,
    /// Statut courant du brouillon.
    #[serde(default = "default_pending_status")]
    pub status: DraftStatus,
    /// Session watcher source (chemin relatif workspace).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "source_session"
    )]
    pub watcher_session: Option<String>,
    /// Contenu structuré du brouillon.
    pub draft: MemoryDraft,
}

impl StoredDraft {
    /// Fabrique un brouillon `pending` avec identifiant et horodatage courants.
    #[must_use]
    pub fn pending(draft: MemoryDraft, watcher_session: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::now_v7().to_string(),
            created_at: Utc::now(),
            status: DraftStatus::Pending,
            watcher_session,
            draft,
        }
    }

    /// Convertit en résumé bridge.
    #[must_use]
    pub fn to_summary(&self) -> DraftSummary {
        DraftSummary {
            id: self.id.clone(),
            title: self.draft.title.clone(),
            kind: self.draft.kind,
            tags: self.draft.tags.clone(),
            status: self.status,
            created_at: self.created_at,
            watcher_session: self.watcher_session.clone(),
        }
    }
}