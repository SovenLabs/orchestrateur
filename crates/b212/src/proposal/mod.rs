//! Propositions trade HITL.

mod builder;
mod hitl;

pub use builder::{build_narrative, build_trade_proposal, determine_side};
pub use hitl::{approve_proposal, mark_sim_executed, reject_proposal};