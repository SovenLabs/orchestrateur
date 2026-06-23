//! Transitions Human-in-the-Loop sur les propositions.

use crate::error::B212Error;
use crate::types::{ProposalStatus, TradeProposal};

/// Approuve une proposition en attente.
///
/// # Errors
///
/// Retourne [`B212Error::InvalidProposalStatus`] si le statut n'est pas `PendingHuman`.
pub fn approve_proposal(mut proposal: TradeProposal) -> Result<TradeProposal, B212Error> {
    if proposal.status != ProposalStatus::PendingHuman {
        return Err(B212Error::InvalidProposalStatus(format!(
            "approve: attendu pending_human, actuel {:?}",
            proposal.status
        )));
    }
    proposal.status = ProposalStatus::HumanApproved;
    Ok(proposal)
}

/// Rejette une proposition en attente.
///
/// # Errors
///
/// Retourne [`B212Error::InvalidProposalStatus`] si le statut n'est pas `PendingHuman`.
pub fn reject_proposal(
    mut proposal: TradeProposal,
    reason: impl Into<String>,
) -> Result<TradeProposal, B212Error> {
    if proposal.status != ProposalStatus::PendingHuman {
        return Err(B212Error::InvalidProposalStatus(format!(
            "reject: attendu pending_human, actuel {:?}",
            proposal.status
        )));
    }
    proposal.status = ProposalStatus::HumanRejected;
    proposal.reject_reason = Some(reason.into());
    Ok(proposal)
}

/// Marque une proposition approuvée comme exécutée en simulation paper.
///
/// # Errors
///
/// Retourne [`B212Error::InvalidProposalStatus`] si le statut n'est pas `HumanApproved`.
pub fn mark_sim_executed(mut proposal: TradeProposal) -> Result<TradeProposal, B212Error> {
    if proposal.status != ProposalStatus::HumanApproved {
        return Err(B212Error::InvalidProposalStatus(format!(
            "sim_execute: attendu human_approved, actuel {:?}",
            proposal.status
        )));
    }
    proposal.status = ProposalStatus::SimExecuted;
    Ok(proposal)
}