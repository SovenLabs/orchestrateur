//! # B212 — Framework liquidity-driven Stratos
//!
//! Crate domaine pur (sans LLM). Orchestrateur reste chef d'orchestre ;
//! ce protocole fournit modules, signaux, scoring et règles auditables.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

pub mod error;
pub mod journal;
pub mod modules;
pub mod ports;
pub mod proposal;
pub mod rules;
pub mod scoring;
pub mod signals;
pub mod types;
pub mod version;

pub use error::B212Error;
pub use journal::{
    entry_cardinal_blocked, entry_proposal_approved, entry_proposal_created,
    entry_proposal_rejected, entry_proposal_sim_executed, entry_setup_analyzed,
};
pub use modules::{
    analyze_b1, analyze_b1_5, analyze_b12, analyze_b2, analyze_b2_5, build_setup_analysis,
    run_all, MacroClimate, MarketRegime, ModuleContext, StructureBias,
};
pub use proposal::{
    approve_proposal, build_narrative, build_trade_proposal, determine_side, mark_sim_executed,
    reject_proposal,
};
pub use rules::evaluate_cardinal_rules;
pub use scoring::{
    build_score_bundle, compute_alignment_score, compute_quick_check, compute_trade_location_score,
    ScoringContext,
};
pub use signals::{
    evaluate_acceptance_expansion, evaluate_cascade_trigger, evaluate_false_migration,
    evaluate_impulse_trigger, evaluate_leverage_trap, evaluate_value_migration, run_all_signals,
    SignalContext,
};
pub use ports::{B212Journal, MarketDataProvider, ProposalRepository};
pub use types::{
    AlignmentScore, B212Lineage, B212SetupAnalysis, Bar, CardinalRuleId, CardinalRulesResult,
    CardinalViolation, JournalEntry, JournalEventKind, MarketScenario, ModuleId, ModuleOutput,
    OhlcvSeries, ProposalStatus, QuickCheckBlock, QuickCheckItem, QuickCheckResult, ScoreBundle,
    SetupContext, SignalKind, SignalOutput, Timeframe, TradeLocationScore, TradeProposal,
};
pub use version::B212_VERSION;