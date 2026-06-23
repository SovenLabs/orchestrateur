//! Types auditables B212 (JSON strict).

mod lineage;
mod market;
mod module_output;
mod proposal;
mod setup;
mod signal;

pub use lineage::B212Lineage;
pub use market::{Bar, MarketScenario, OhlcvSeries, Timeframe};
pub use module_output::{ModuleId, ModuleOutput};
pub use proposal::{ProposalStatus, TradeProposal};
pub use setup::{B212SetupAnalysis, SetupContext};
pub use signal::{SignalKind, SignalOutput};