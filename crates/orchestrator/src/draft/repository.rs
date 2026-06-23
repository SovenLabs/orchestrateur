//! Port de persistance des brouillons insight.

use async_trait::async_trait;
use cortex::MemoryDraft;
use thiserror::Error;

use super::record::{DraftStatus, StoredDraft};

/// Erreur de persistance des brouillons.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DraftError {
    /// Brouillon introuvable.
    #[error("brouillon introuvable: {0}")]
    NotFound(String),

    /// Transition de statut invalide.
    #[error("statut invalide: attendu {expected}, actuel {actual}")]
    InvalidStatus {
        /// Statut attendu.
        expected: &'static str,
        /// Statut actuel.
        actual: &'static str,
    },

    /// Erreur I/O disque.
    #[error("I/O: {0}")]
    Io(String),

    /// Erreur de sérialisation JSON.
    #[error("sérialisation: {0}")]
    Serialization(String),
}

/// Port de persistance des brouillons (`workspace/.orchestrateur/drafts/{uuid}.json`).
///
/// Implémenté par `infrastructure::FileDraftRepository` ou mocks de test.
#[async_trait]
pub trait DraftRepository: Send + Sync {
    /// Persiste un brouillon (création ou mise à jour).
    async fn save(&self, stored: &StoredDraft) -> Result<(), DraftError>;

    /// Crée et persiste un brouillon `pending`.
    async fn create_pending(
        &self,
        draft: MemoryDraft,
        watcher_session: Option<String>,
    ) -> Result<StoredDraft, DraftError> {
        let stored = StoredDraft::pending(draft, watcher_session);
        self.save(&stored).await?;
        Ok(stored)
    }

    /// Charge un brouillon par identifiant.
    async fn get_by_id(&self, id: &str) -> Result<StoredDraft, DraftError>;

    /// Liste les brouillons, filtrés optionnellement par statut.
    async fn list(&self, status: Option<DraftStatus>) -> Result<Vec<StoredDraft>, DraftError>;

    /// Met à jour le statut d'un brouillon et le réécrit sur disque.
    async fn update_status(
        &self,
        id: &str,
        status: DraftStatus,
    ) -> Result<StoredDraft, DraftError>;
}