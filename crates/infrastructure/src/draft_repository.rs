use std::path::{Path, PathBuf};

use async_trait::async_trait;
use orchestrator::draft::{DraftError, DraftRepository, DraftStatus, StoredDraft};
use tokio::fs;

/// Persistance JSON des brouillons (`workspace/.orchestrateur/drafts/{uuid}.json`).
pub struct FileDraftRepository {
    drafts_dir: PathBuf,
}

impl FileDraftRepository {
    /// Crée un dépôt fichier pointant vers `drafts_dir`.
    pub fn new(drafts_dir: impl Into<PathBuf>) -> Self {
        Self {
            drafts_dir: drafts_dir.into(),
        }
    }

    fn path_for(&self, id: &str) -> PathBuf {
        self.drafts_dir.join(format!("{id}.json"))
    }

    async fn ensure_dir(&self) -> Result<(), DraftError> {
        fs::create_dir_all(&self.drafts_dir)
            .await
            .map_err(|e| DraftError::Io(format!("création répertoire drafts: {e}")))?;
        Ok(())
    }

    async fn read_file(&self, path: &Path) -> Result<StoredDraft, DraftError> {
        let raw = fs::read_to_string(path)
            .await
            .map_err(|e| DraftError::Io(format!("lecture {}: {e}", path.display())))?;
        serde_json::from_str(&raw).map_err(|e| DraftError::Serialization(e.to_string()))
    }
}

#[async_trait]
impl DraftRepository for FileDraftRepository {
    async fn save(&self, stored: &StoredDraft) -> Result<(), DraftError> {
        self.ensure_dir().await?;
        let path = self.path_for(&stored.id);
        let json = serde_json::to_string_pretty(stored)
            .map_err(|e| DraftError::Serialization(e.to_string()))?;
        fs::write(&path, json)
            .await
            .map_err(|e| DraftError::Io(format!("écriture {}: {e}", path.display())))?;
        tracing::debug!(path = %path.display(), draft_id = %stored.id, "brouillon persisté");
        Ok(())
    }

    async fn get_by_id(&self, id: &str) -> Result<StoredDraft, DraftError> {
        let path = self.path_for(id);
        if !path.exists() {
            return Err(DraftError::NotFound(id.to_string()));
        }
        self.read_file(&path).await
    }

    async fn list(&self, status: Option<DraftStatus>) -> Result<Vec<StoredDraft>, DraftError> {
        self.ensure_dir().await?;
        let mut entries = Vec::new();
        let mut read_dir = fs::read_dir(&self.drafts_dir)
            .await
            .map_err(|e| DraftError::Io(format!("listage drafts: {e}")))?;

        while let Some(entry) = read_dir
            .next_entry()
            .await
            .map_err(|e| DraftError::Io(e.to_string()))?
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(stored) = self.read_file(&path).await {
                if status.is_none_or(|s| stored.status == s) {
                    entries.push(stored);
                }
            }
        }
        entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(entries)
    }

    async fn update_status(
        &self,
        id: &str,
        status: DraftStatus,
    ) -> Result<StoredDraft, DraftError> {
        let mut stored = self.get_by_id(id).await?;
        stored.status = status;
        self.save(&stored).await?;
        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use cortex::{MemoryDraft, MemoryKind};

    use super::*;

    #[tokio::test]
    async fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let repo = FileDraftRepository::new(dir.path());

        let mut draft = MemoryDraft::new("Titre FS", "Contenu persistant.");
        draft.kind = MemoryKind::Decision;
        draft.tags = vec!["rust".into()];

        let stored = repo
            .create_pending(draft, Some("sessions/foo.md".into()))
            .await
            .unwrap();
        assert_eq!(stored.status, DraftStatus::Pending);
        assert!(dir.path().join(format!("{}.json", stored.id)).exists());

        let loaded = repo.get_by_id(&stored.id).await.unwrap();
        assert_eq!(loaded.draft.title, "Titre FS");
        assert_eq!(loaded.watcher_session.as_deref(), Some("sessions/foo.md"));
    }

    #[tokio::test]
    async fn list_filters_by_status() {
        let dir = tempfile::tempdir().unwrap();
        let repo = FileDraftRepository::new(dir.path());

        let a = repo
            .create_pending(MemoryDraft::new("A", "a"), None)
            .await
            .unwrap();
        let b = repo
            .create_pending(MemoryDraft::new("B", "b"), None)
            .await
            .unwrap();
        repo.update_status(&b.id, DraftStatus::Discarded)
            .await
            .unwrap();

        assert_eq!(
            repo.list(Some(DraftStatus::Pending)).await.unwrap().len(),
            1
        );
        assert_eq!(repo.list(None).await.unwrap().len(), 2);
        assert_eq!(a.id, repo.list(Some(DraftStatus::Pending)).await.unwrap()[0].id);
    }

    #[tokio::test]
    async fn missing_draft_returns_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let repo = FileDraftRepository::new(dir.path());
        assert!(matches!(
            repo.get_by_id("missing").await,
            Err(DraftError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn update_status_persists() {
        let dir = tempfile::tempdir().unwrap();
        let repo = FileDraftRepository::new(dir.path());
        let stored = repo
            .create_pending(MemoryDraft::new("X", "x"), None)
            .await
            .unwrap();
        let updated = repo
            .update_status(&stored.id, DraftStatus::Published)
            .await
            .unwrap();
        assert_eq!(updated.status, DraftStatus::Published);
        let reloaded = repo.get_by_id(&stored.id).await.unwrap();
        assert_eq!(reloaded.status, DraftStatus::Published);
    }
}