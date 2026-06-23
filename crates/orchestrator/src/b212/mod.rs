//! Intégration B212 dans Orchestrateur (workflow, bridge — PR-6+).

pub use b212::{
    analyze_b1, analyze_b1_5, analyze_b12, analyze_b2, analyze_b2_5, build_setup_analysis,
    run_all, run_all_signals, B212Error, B212Lineage, B212SetupAnalysis, B212_VERSION, MacroClimate,
    MarketDataProvider, MarketRegime, ModuleContext, OhlcvSeries, SetupContext, SignalKind,
    SignalOutput, StructureBias, Timeframe, TradeProposal,
};