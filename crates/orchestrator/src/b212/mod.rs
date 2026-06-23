//! Intégration B212 dans Orchestrateur (workflow, bridge — PR-6+).

mod governance;

pub use b212::{
    analyze_b1, analyze_b1_5, analyze_b12, analyze_b2, analyze_b2_5, approve_proposal,
    build_narrative, build_score_bundle, build_setup_analysis, build_trade_proposal,
    compute_alignment_score, compute_quick_check, compute_trade_location_score,
    entry_cardinal_blocked, entry_proposal_approved, entry_proposal_created,
    entry_proposal_rejected, entry_proposal_sim_executed, entry_setup_analyzed,
    evaluate_cardinal_rules, mark_sim_executed, reject_proposal, run_all, run_all_signals,
    AlignmentScore, B212Error, B212Journal, B212Lineage, B212SetupAnalysis, B212_VERSION,
    CardinalRuleId, CardinalRulesResult, CardinalViolation, JournalEntry, JournalEventKind,
    MacroClimate, MarketDataProvider, MarketRegime, ModuleContext, OhlcvSeries, ProposalRepository,
    ProposalStatus, QuickCheckResult, ScoreBundle, ScoringContext, SetupContext, SignalKind,
    SignalOutput, StructureBias, Timeframe, TradeLocationScore, TradeProposal,
};
pub use governance::B212GovernanceService;