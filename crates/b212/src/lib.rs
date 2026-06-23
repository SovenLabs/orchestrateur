//! # B212 — Framework liquidity-driven Stratos
//!
//! Crate domaine pur (sans LLM). Orchestrateur reste chef d'orchestre ;
//! ce protocole fournit modules, signaux, scoring et règles auditables.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

pub mod error;
pub mod modules;
pub mod ports;
pub mod types;
pub mod version;

pub use error::B212Error;
pub use modules::{
    analyze_b1, analyze_b1_5, analyze_b12, analyze_b2, analyze_b2_5, build_setup_analysis,
    run_all, MacroClimate, MarketRegime, ModuleContext, StructureBias,
};
pub use ports::MarketDataProvider;
pub use types::{
    B212Lineage, B212SetupAnalysis, Bar, MarketScenario, ModuleId, ModuleOutput, OhlcvSeries,
    ProposalStatus, SetupContext, Timeframe, TradeProposal,
};
pub use version::B212_VERSION;