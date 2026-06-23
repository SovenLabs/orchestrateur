//! Helpers de construction d'entrées journal B212.

use chrono::Utc;
use serde_json::json;

use crate::types::{
    B212Lineage, B212SetupAnalysis, CardinalRulesResult, JournalEntry, JournalEventKind,
    TradeProposal,
};

/// Entrée pour une analyse setup complétée.
#[must_use]
pub fn entry_setup_analyzed(analysis: &B212SetupAnalysis) -> JournalEntry {
    JournalEntry {
        timestamp: Utc::now().to_rfc3339(),
        event_kind: JournalEventKind::SetupAnalyzed,
        symbol: analysis.symbol.clone(),
        session: analysis.session.clone(),
        proposal_id: None,
        details: json!({
            "module_count": analysis.modules.len(),
            "signal_count": analysis.signals.len(),
            "cardinal_passed": analysis.cardinal.as_ref().map(|c| c.passed),
            "recommended_sizing": analysis.scores.as_ref().map(|s| &s.recommended_sizing),
            "trade_location": analysis.scores.as_ref().map(|s| s.trade_location.total),
            "alignment": analysis.scores.as_ref().map(|s| s.alignment.total),
        }),
        lineage: analysis.lineage.clone(),
    }
}

/// Entrée pour blocage cardinal.
#[must_use]
pub fn entry_cardinal_blocked(
    analysis: &B212SetupAnalysis,
    cardinal: &CardinalRulesResult,
) -> JournalEntry {
    JournalEntry {
        timestamp: Utc::now().to_rfc3339(),
        event_kind: JournalEventKind::CardinalBlocked,
        symbol: analysis.symbol.clone(),
        session: analysis.session.clone(),
        proposal_id: None,
        details: json!({
            "violations": cardinal.violations,
            "rationale": cardinal.rationale,
        }),
        lineage: analysis.lineage.clone(),
    }
}

/// Entrée pour création de proposition.
#[must_use]
pub fn entry_proposal_created(proposal: &TradeProposal) -> JournalEntry {
    JournalEntry {
        timestamp: Utc::now().to_rfc3339(),
        event_kind: JournalEventKind::ProposalCreated,
        symbol: proposal.symbol.clone(),
        session: proposal.session.clone(),
        proposal_id: Some(proposal.id.clone()),
        details: json!({
            "side": proposal.side,
            "sizing": proposal.sizing,
            "trade_location_score": proposal.trade_location_score,
            "alignment_score": proposal.alignment_score,
            "quick_check_passed": proposal.quick_check_passed,
        }),
        lineage: proposal.lineage.clone(),
    }
}

/// Entrée pour approbation HITL.
#[must_use]
pub fn entry_proposal_approved(proposal: &TradeProposal) -> JournalEntry {
    journal_proposal_event(proposal, JournalEventKind::ProposalApproved, json!({}))
}

/// Entrée pour rejet HITL.
#[must_use]
pub fn entry_proposal_rejected(proposal: &TradeProposal) -> JournalEntry {
    journal_proposal_event(
        proposal,
        JournalEventKind::ProposalRejected,
        json!({ "reject_reason": proposal.reject_reason }),
    )
}

/// Entrée pour exécution simulation paper.
#[must_use]
pub fn entry_proposal_sim_executed(proposal: &TradeProposal) -> JournalEntry {
    journal_proposal_event(proposal, JournalEventKind::ProposalSimExecuted, json!({}))
}

fn journal_proposal_event(
    proposal: &TradeProposal,
    kind: JournalEventKind,
    details: serde_json::Value,
) -> JournalEntry {
    JournalEntry {
        timestamp: Utc::now().to_rfc3339(),
        event_kind: kind,
        symbol: proposal.symbol.clone(),
        session: proposal.session.clone(),
        proposal_id: Some(proposal.id.clone()),
        details,
        lineage: proposal.lineage.clone(),
    }
}

/// Lignée fixture pour entrées journal de test.
#[must_use]
pub fn fixture_lineage() -> B212Lineage {
    B212Lineage::fixture("b212_journal")
}