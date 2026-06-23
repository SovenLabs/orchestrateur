//! Handlers bridge pour les commandes B212.

use b212::{ProposalStatus, TradeProposal};

use crate::bridge::{
    AppError, B212AgentStepSummary, B212ProposalSummary, B212WorkflowSummary, Response,
};
use crate::facade::OrchestratorFacade;

use super::workflow::{B212AnalyzeRequest, B212WorkflowResult};

/// Initialise les agents domaine B212.
pub async fn execute_b212_init_agents(facade: &OrchestratorFacade) -> Response {
    match facade.b212_init_agents().await {
        Ok(ids) => Response::B212AgentsReady { agent_ids: ids },
        Err(err) => b212_agent_error(err),
    }
}

/// Exécute le workflow d'analyse B212.
pub async fn execute_b212_analyze(
    facade: &OrchestratorFacade,
    symbol: &str,
    session: &str,
    lookback: usize,
) -> Response {
    let req = B212AnalyzeRequest {
        symbol: symbol.to_string(),
        session: session.to_string(),
        lookback,
    };
    match facade.b212_analyze(req).await {
        Ok(result) => Response::B212Workflow {
            result: workflow_summary(&result),
        },
        Err(err) => b212_error(err),
    }
}

/// Liste les propositions en attente.
pub async fn execute_b212_list_proposals(facade: &OrchestratorFacade) -> Response {
    match facade.b212_list_pending_proposals().await {
        Ok(items) => Response::B212ProposalList {
            items: items.iter().map(proposal_summary).collect(),
        },
        Err(err) => b212_error(err),
    }
}

/// Approuve une proposition.
pub async fn execute_b212_approve_proposal(facade: &OrchestratorFacade, id: &str) -> Response {
    match facade.b212_approve_proposal(id).await {
        Ok(p) => Response::B212ProposalUpdated {
            proposal: proposal_summary(&p),
        },
        Err(err) => b212_error(err),
    }
}

/// Rejette une proposition.
pub async fn execute_b212_reject_proposal(
    facade: &OrchestratorFacade,
    id: &str,
    reason: &str,
) -> Response {
    match facade.b212_reject_proposal(id, reason).await {
        Ok(p) => Response::B212ProposalUpdated {
            proposal: proposal_summary(&p),
        },
        Err(err) => b212_error(err),
    }
}

fn workflow_summary(result: &B212WorkflowResult) -> B212WorkflowSummary {
    let scores = result.analysis.scores.as_ref();
    let cardinal_passed = result
        .analysis
        .cardinal
        .as_ref()
        .is_some_and(|c| c.passed);
    B212WorkflowSummary {
        symbol: result.symbol.clone(),
        session: result.session.clone(),
        step_count: result.steps.len(),
        cardinal_passed,
        recommended_sizing: scores
            .map(|s| s.recommended_sizing.clone())
            .unwrap_or_else(|| "none".into()),
        trade_location_score: scores.map(|s| s.trade_location.total).unwrap_or(0),
        alignment_score: scores.map(|s| s.alignment.total).unwrap_or(0),
        proposal_id: result.proposal.as_ref().map(|p| p.id.clone()),
        steps: result
            .steps
            .iter()
            .map(|s| B212AgentStepSummary {
                agent_id: s.agent_id.clone(),
                agent_name: s.agent_name.clone(),
                summary: s.summary.clone(),
            })
            .collect(),
    }
}

fn proposal_summary(p: &TradeProposal) -> B212ProposalSummary {
    B212ProposalSummary {
        id: p.id.clone(),
        symbol: p.symbol.clone(),
        session: p.session.clone(),
        side: p.side.clone(),
        status: proposal_status_label(p.status),
        trade_location_score: p.trade_location_score,
        sizing: p.sizing.clone(),
    }
}

fn proposal_status_label(status: ProposalStatus) -> String {
    match status {
        ProposalStatus::PendingHuman => "pending_human",
        ProposalStatus::HumanApproved => "human_approved",
        ProposalStatus::HumanRejected => "human_rejected",
        ProposalStatus::SimExecuted => "sim_executed",
    }
    .into()
}

fn b212_error(err: b212::B212Error) -> Response {
    Response::Error(AppError {
        kind: "b212".into(),
        message: err.to_string(),
    })
}

fn b212_agent_error(err: crate::persistent::PersistentAgentError) -> Response {
    Response::Error(AppError {
        kind: "b212_agent".into(),
        message: err.to_string(),
    })
}