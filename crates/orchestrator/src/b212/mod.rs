//! Intégration B212 dans Orchestrateur (workflow, bridge — PR-6+).

pub use b212::{
    B212Error, B212Lineage, B212SetupAnalysis, B212_VERSION, MarketDataProvider, OhlcvSeries,
    SetupContext, Timeframe, TradeProposal,
};