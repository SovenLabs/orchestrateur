//! Port persistance propositions HITL.

use async_trait::async_trait;

use crate::error::B212Error;
use crate::types::{ProposalStatus, TradeProposal};

/// Persistance des propositions trade (`workspace/b212/proposals/`).
#[async_trait]
pub trait ProposalRepository: Send + Sync {
    /// Sauvegarde une proposition (création ou mise à jour).
    async fn save(&self, proposal: &TradeProposal) -> Result<(), B212Error>;

    /// Charge une proposition par identifiant.
    async fn get(&self, id: &str) -> Result<TradeProposal, B212Error>;

    /// Liste les propositions en attente validation humaine.
    async fn list_pending(&self) -> Result<Vec<TradeProposal>, B212Error>;

    /// Met à jour le statut d'une proposition.
    async fn update_status(
        &self,
        id: &str,
        status: ProposalStatus,
        reject_reason: Option<String>,
    ) -> Result<TradeProposal, B212Error>;
}