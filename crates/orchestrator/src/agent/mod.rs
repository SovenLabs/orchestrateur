//! Boucle agentique Phase 7 — [`AgentLoop`] + v2 ports Cortex.

pub mod adapters;

mod config;
mod context;
mod error;
mod loop_impl;
mod loop_v2;
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
pub use loop_v2::{AgentLoopV2, AgentResponse};
pub use stream::{AgentStreamEvent, AgentStreamSink};