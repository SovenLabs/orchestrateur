//! Gouvernance B212 — journal JSONL, propositions HITL (PR-5).

use std::sync::Arc;

use b212::{
    approve_proposal, build_trade_proposal, entry_cardinal_blocked, entry_proposal_approved,
    entry_proposal_created, entry_proposal_rejected, entry_proposal_sim_executed,
    entry_setup_analyzed, mark_sim_executed, reject_proposal, B212Error, B212Journal,
    B212SetupAnalysis, ProposalRepository, TradeProposal,
};

/// Service de gouvernance B212 (journal + propositions).
pub struct B212GovernanceService {
    journal: Arc<dyn B212Journal>,
    proposals: Arc<dyn ProposalRepository>,
}

impl B212GovernanceService {
    /// Construit le service avec journal et dépôt propositions.
    pub fn new(journal: Arc<dyn B212Journal>, proposals: Arc<dyn ProposalRepository>) -> Self {
        Self { journal, proposals }
    }

    /// Journalise une analyse et tente de créer une proposition HITL.
    pub async fn process_analysis(
        &self,
        analysis: &B212SetupAnalysis,
    ) -> Result<Option<TradeProposal>, B212Error> {
        self.journal
            .append(&entry_setup_analyzed(analysis))
            .await?;

        if let Some(cardinal) = analysis.cardinal.as_ref().filter(|c| !c.passed) {
            self.journal
                .append(&entry_cardinal_blocked(analysis, cardinal))
                .await?;
            return Ok(None);
        }

        match build_trade_proposal(analysis) {
            Ok(proposal) => {
                self.proposals.save(&proposal).await?;
                self.journal
                    .append(&entry_proposal_created(&proposal))
                    .await?;
                Ok(Some(proposal))
            }
            Err(B212Error::ProposalBlocked(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Approuve une proposition en attente.
    pub async fn approve(&self, id: &str) -> Result<TradeProposal, B212Error> {
        let proposal = self.proposals.get(id).await?;
        let approved = approve_proposal(proposal)?;
        self.proposals.save(&approved).await?;
        self.journal
            .append(&entry_proposal_approved(&approved))
            .await?;
        Ok(approved)
    }

    /// Rejette une proposition en attente.
    pub async fn reject(&self, id: &str, reason: &str) -> Result<TradeProposal, B212Error> {
        let proposal = self.proposals.get(id).await?;
        let rejected = reject_proposal(proposal, reason)?;
        self.proposals.save(&rejected).await?;
        self.journal
            .append(&entry_proposal_rejected(&rejected))
            .await?;
        Ok(rejected)
    }

    /// Marque une proposition approuvée comme exécutée en simulation paper.
    pub async fn sim_execute(&self, id: &str) -> Result<TradeProposal, B212Error> {
        let proposal = self.proposals.get(id).await?;
        let executed = mark_sim_executed(proposal)?;
        self.proposals.save(&executed).await?;
        self.journal
            .append(&entry_proposal_sim_executed(&executed))
            .await?;
        Ok(executed)
    }

    /// Liste les propositions en attente validation humaine.
    pub async fn list_pending(&self) -> Result<Vec<TradeProposal>, B212Error> {
        self.proposals.list_pending().await
    }
}