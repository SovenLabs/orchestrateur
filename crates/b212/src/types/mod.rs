//! Types auditables B212 (JSON strict).

mod cardinal;
mod journal;
mod lineage;
mod market;
mod module_output;
mod proposal;
mod score;
mod setup;
mod signal;

pub use cardinal::{CardinalRuleId, CardinalRulesResult, CardinalViolation};
pub use journal::{JournalEntry, JournalEventKind};
pub use lineage::B212Lineage;
pub use market::{Bar, MarketScenario, OhlcvSeries, Timeframe};
pub use module_output::{ModuleId, ModuleOutput};
pub use proposal::{ProposalStatus, TradeProposal};
pub use score::{
    AlignmentScore, QuickCheckBlock, QuickCheckItem, QuickCheckResult, ScoreBundle,
    TradeLocationScore,
};
pub use setup::{B212SetupAnalysis, SetupContext};
pub use signal::{SignalKind, SignalOutput};