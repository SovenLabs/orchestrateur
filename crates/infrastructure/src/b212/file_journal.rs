use std::path::PathBuf;

use async_trait::async_trait;
use b212::{B212Error, B212Journal, JournalEntry};
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;

/// Journal B212 append-only (`workspace/b212/journal/audit.jsonl`).
pub struct FileB212Journal {
    path: PathBuf,
}

impl FileB212Journal {
    /// Ouvre le journal à `journal_dir/audit.jsonl`.
    pub fn new(journal_dir: impl Into<PathBuf>) -> Self {
        Self {
            path: journal_dir.into().join("audit.jsonl"),
        }
    }
}

#[async_trait]
impl B212Journal for FileB212Journal {
    async fn append(&self, entry: &JournalEntry) -> Result<(), B212Error> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                B212Error::Journal(format!("création répertoire {}: {e}", parent.display()))
            })?;
        }
        let line = serde_json::to_string(entry)
            .map_err(|e| B212Error::Journal(format!("sérialisation entrée: {e}")))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await
            .map_err(|e| B212Error::Journal(format!("ouverture {}: {e}", self.path.display())))?;
        file.write_all(line.as_bytes())
            .await
            .map_err(|e| B212Error::Journal(format!("écriture {}: {e}", self.path.display())))?;
        file.write_all(b"\n")
            .await
            .map_err(|e| B212Error::Journal(format!("écriture newline: {e}")))?;
        Ok(())
    }
}