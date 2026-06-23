//! Gouvernance B212 — journal JSONL, propositions HITL (PR-5+).

use std::sync::Arc;

use b212::{
    approve_proposal, build_trade_proposal, entry_cardinal_blocked, entry_proposal_approved,
    entry_proposal_created, entry_proposal_rejected, entry_proposal_sim_executed,
    entry_setup_analyzed, mark_sim_executed, reject_proposal, B212Error, B212Journal,
    B212SetupAnalysis, ProposalRepository, SimFill, TradeProposal,
};

use crate::events::{B212Event, EventPublisher};

/// Service de gouvernance B212 (journal + propositions).
pub struct B212GovernanceService {
    journal: Arc<dyn B212Journal>,
    proposals: Arc<dyn ProposalRepository>,
    events: Option<Arc<dyn EventPublisher>>,
    events_enabled: bool,
}

impl B212GovernanceService {
    /// Construit le service avec journal et dépôt propositions.
    pub fn new(journal: Arc<dyn B212Journal>, proposals: Arc<dyn ProposalRepository>) -> Self {
        Self {
            journal,
            proposals,
            events: None,
            events_enabled: false,
        }
    }

    /// Active la publication d'événements B212 optionnels.
    pub fn with_events(
        journal: Arc<dyn B212Journal>,
        proposals: Arc<dyn ProposalRepository>,
        events: Arc<dyn EventPublisher>,
        events_enabled: bool,
    ) -> Self {
        Self {
            journal,
            proposals,
            events: Some(events),
            events_enabled,
        }
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
            self.emit(B212Event::AnalysisComplete {
                symbol: analysis.symbol.clone(),
                session: analysis.session.clone(),
                proposal_id: None,
            });
            return Ok(None);
        }

        match build_trade_proposal(analysis) {
            Ok(proposal) => {
                self.proposals.save(&proposal).await?;
                self.journal
                    .append(&entry_proposal_created(&proposal))
                    .await?;
                self.emit(B212Event::ProposalCreated {
                    proposal_id: proposal.id.clone(),
                    symbol: proposal.symbol.clone(),
                    side: proposal.side.clone(),
                });
                self.emit(B212Event::AnalysisComplete {
                    symbol: analysis.symbol.clone(),
                    session: analysis.session.clone(),
                    proposal_id: Some(proposal.id.clone()),
                });
                Ok(Some(proposal))
            }
            Err(B212Error::ProposalBlocked(_)) => {
                self.emit(B212Event::AnalysisComplete {
                    symbol: analysis.symbol.clone(),
                    session: analysis.session.clone(),
                    proposal_id: None,
                });
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// Charge une proposition par identifiant.
    pub async fn get_proposal(&self, id: &str) -> Result<TradeProposal, B212Error> {
        self.proposals.get(id).await
    }

    /// Persiste une proposition (tests / intégration).
    pub async fn register_proposal(&self, proposal: &TradeProposal) -> Result<(), B212Error> {
        self.proposals.save(proposal).await?;
        self.journal
            .append(&entry_proposal_created(proposal))
            .await?;
        self.emit(B212Event::ProposalCreated {
            proposal_id: proposal.id.clone(),
            symbol: proposal.symbol.clone(),
            side: proposal.side.clone(),
        });
        Ok(())
    }

    /// Approuve une proposition en attente.
    pub async fn approve(&self, id: &str) -> Result<TradeProposal, B212Error> {
        let proposal = self.proposals.get(id).await?;
        let approved = approve_proposal(proposal)?;
        self.proposals.save(&approved).await?;
        self.journal
            .append(&entry_proposal_approved(&approved))
            .await?;
        self.emit(B212Event::ProposalApproved {
            proposal_id: approved.id.clone(),
        });
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
        self.emit(B212Event::ProposalRejected {
            proposal_id: rejected.id.clone(),
        });
        Ok(rejected)
    }

    /// Marque une proposition approuvée comme exécutée en simulation paper.
    pub async fn sim_execute(&self, id: &str) -> Result<TradeProposal, B212Error> {
        self.sim_execute_with_fill(id, None).await
    }

    /// Marque exécutée avec détails fill optionnels.
    pub async fn sim_execute_with_fill(
        &self,
        id: &str,
        fill: Option<&SimFill>,
    ) -> Result<TradeProposal, B212Error> {
        let proposal = self.proposals.get(id).await?;
        let executed = mark_sim_executed(proposal)?;
        self.proposals.save(&executed).await?;
        self.journal
            .append(&entry_proposal_sim_executed(&executed, fill))
            .await?;
        if let Some(f) = fill {
            self.emit(B212Event::SimExecuted {
                proposal_id: executed.id.clone(),
                fill_id: f.id.clone(),
                entry_price: f.entry_price,
                notional_usd: f.notional_usd,
            });
        }
        Ok(executed)
    }

    /// Liste les propositions en attente validation humaine.
    pub async fn list_pending(&self) -> Result<Vec<TradeProposal>, B212Error> {
        self.proposals.list_pending().await
    }

    fn emit(&self, event: B212Event) {
        if self.events_enabled {
            if let Some(events) = &self.events {
                events.publish_b212(&[event]);
            }
        }
    }
}