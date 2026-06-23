//! Intégration B212 dans Orchestrateur (workflow, bridge — PR-6+).

pub use b212::{
    analyze_b1, analyze_b1_5, analyze_b12, analyze_b2, analyze_b2_5, build_score_bundle,
    build_setup_analysis, compute_alignment_score, compute_quick_check, compute_trade_location_score,
    run_all, run_all_signals, AlignmentScore, B212Error, B212Lineage, B212SetupAnalysis, B212_VERSION,
    MacroClimate, MarketDataProvider, MarketRegime, ModuleContext, OhlcvSeries, QuickCheckResult,
    ScoreBundle, ScoringContext, SetupContext, SignalKind, SignalOutput, StructureBias, Timeframe,
    TradeLocationScore, TradeProposal,
};