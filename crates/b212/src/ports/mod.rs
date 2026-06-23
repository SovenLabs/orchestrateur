//! Ports hexagonaux B212 (adapters dans `infrastructure`).

mod journal;
mod market_data;
mod proposal_store;

pub use journal::B212Journal;
pub use market_data::MarketDataProvider;
pub use proposal_store::ProposalRepository;