//! Boucle agentique — [`AgentLoop`] + adapters ports Cortex.

pub mod adapters;

mod config;
mod context;
mod error;
mod loop_impl;
mod message_preprocessor;
mod stream;
mod tool_parse;

pub use adapters::{
    agent_exchange_turn, build_agent_adapters, ChangeDetector, ChangeDetectorConfig,
    CortexAssimilationService, CortexContextProvider, CortexSemanticSearch,
};
pub use config::AgentConfig;
pub use error::AgentError;
pub use loop_impl::{AgentLoop, AgentTurnRequest, AgentTurnResult};
pub use stream::{AgentStreamEvent, AgentStreamSink};