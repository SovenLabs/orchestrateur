use std::path::{Path, PathBuf};

use async_trait::async_trait;
use b212::{B212Error, ProposalRepository, ProposalStatus, TradeProposal};
use tokio::fs;

/// Persistance JSON des propositions (`workspace/b212/proposals/{id}.json`).
pub struct FileProposalRepository {
    proposals_dir: PathBuf,
}

impl FileProposalRepository {
    /// Crée un dépôt fichier pointant vers `proposals_dir`.
    pub fn new(proposals_dir: impl Into<PathBuf>) -> Self {
        Self {
            proposals_dir: proposals_dir.into(),
        }
    }

    fn path_for(&self, id: &str) -> PathBuf {
        self.proposals_dir.join(format!("{id}.json"))
    }

    async fn ensure_dir(&self) -> Result<(), B212Error> {
        fs::create_dir_all(&self.proposals_dir)
            .await
            .map_err(|e| B212Error::Journal(format!("création proposals: {e}")))?;
        Ok(())
    }

    async fn read_file(&self, path: &Path) -> Result<TradeProposal, B212Error> {
        let raw = fs::read_to_string(path)
            .await
            .map_err(|e| B212Error::Journal(format!("lecture {}: {e}", path.display())))?;
        serde_json::from_str(&raw).map_err(|e| B212Error::Parse(e.to_string()))
    }
}

#[async_trait]
impl ProposalRepository for FileProposalRepository {
    async fn save(&self, proposal: &TradeProposal) -> Result<(), B212Error> {
        self.ensure_dir().await?;
        let path = self.path_for(&proposal.id);
        let json = serde_json::to_string_pretty(proposal)
            .map_err(|e| B212Error::Parse(e.to_string()))?;
        fs::write(&path, json)
            .await
            .map_err(|e| B212Error::Journal(format!("écriture {}: {e}", path.display())))?;
        Ok(())
    }

    async fn get(&self, id: &str) -> Result<TradeProposal, B212Error> {
        let path = self.path_for(id);
        if !path.exists() {
            return Err(B212Error::ProposalNotFound(id.to_string()));
        }
        self.read_file(&path).await
    }

    async fn list_pending(&self) -> Result<Vec<TradeProposal>, B212Error> {
        self.ensure_dir().await?;
        let mut proposals = Vec::new();
        let mut read_dir = fs::read_dir(&self.proposals_dir)
            .await
            .map_err(|e| B212Error::Journal(format!("listage proposals: {e}")))?;
        while let Some(entry) = read_dir
            .next_entry()
            .await
            .map_err(|e| B212Error::Journal(e.to_string()))?
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(proposal) = self.read_file(&path).await {
                if proposal.status == ProposalStatus::PendingHuman {
                    proposals.push(proposal);
                }
            }
        }
        proposals.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(proposals)
    }

    async fn update_status(
        &self,
        id: &str,
        status: ProposalStatus,
        reject_reason: Option<String>,
    ) -> Result<TradeProposal, B212Error> {
        let mut proposal = self.get(id).await?;
        proposal.status = status;
        proposal.reject_reason = reject_reason;
        self.save(&proposal).await?;
        Ok(proposal)
    }
}