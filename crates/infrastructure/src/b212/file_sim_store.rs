use std::path::{Path, PathBuf};

use async_trait::async_trait;
use b212::{B212Error, SimFill, SimTradeRepository};
use tokio::fs;

/// Persistance JSON des fills paper (`workspace/b212/sim/{id}.json`).
pub struct FileSimTradeRepository {
    sim_dir: PathBuf,
}

impl FileSimTradeRepository {
    /// Crée un dépôt fichier pointant vers `sim_dir`.
    pub fn new(sim_dir: impl Into<PathBuf>) -> Self {
        Self {
            sim_dir: sim_dir.into(),
        }
    }

    fn path_for(&self, id: &str) -> PathBuf {
        self.sim_dir.join(format!("{id}.json"))
    }

    async fn ensure_dir(&self) -> Result<(), B212Error> {
        fs::create_dir_all(&self.sim_dir)
            .await
            .map_err(|e| B212Error::Journal(format!("création sim: {e}")))?;
        Ok(())
    }

    async fn read_file(&self, path: &Path) -> Result<SimFill, B212Error> {
        let raw = fs::read_to_string(path)
            .await
            .map_err(|e| B212Error::Journal(format!("lecture {}: {e}", path.display())))?;
        serde_json::from_str(&raw).map_err(|e| B212Error::Parse(e.to_string()))
    }
}

#[async_trait]
impl SimTradeRepository for FileSimTradeRepository {
    async fn save(&self, fill: &SimFill) -> Result<(), B212Error> {
        self.ensure_dir().await?;
        let path = self.path_for(&fill.id);
        let json = serde_json::to_string_pretty(fill)
            .map_err(|e| B212Error::Parse(e.to_string()))?;
        fs::write(&path, json)
            .await
            .map_err(|e| B212Error::Journal(format!("écriture {}: {e}", path.display())))?;
        Ok(())
    }

    async fn get(&self, id: &str) -> Result<SimFill, B212Error> {
        let path = self.path_for(id);
        if !path.exists() {
            return Err(B212Error::SimFillNotFound(id.to_string()));
        }
        self.read_file(&path).await
    }

    async fn list_for_proposal(&self, proposal_id: &str) -> Result<Vec<SimFill>, B212Error> {
        self.ensure_dir().await?;
        let mut fills = Vec::new();
        let mut read_dir = fs::read_dir(&self.sim_dir)
            .await
            .map_err(|e| B212Error::Journal(format!("listage sim: {e}")))?;
        while let Some(entry) = read_dir
            .next_entry()
            .await
            .map_err(|e| B212Error::Journal(e.to_string()))?
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(fill) = self.read_file(&path).await {
                if fill.proposal_id == proposal_id {
                    fills.push(fill);
                }
            }
        }
        fills.sort_by(|a, b| a.fill_ts.cmp(&b.fill_ts));
        Ok(fills)
    }
}