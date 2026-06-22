//! Boucle agentique Phase 7 — [`AgentLoop`].

mod config;
mod context;
mod error;
mod loop_impl;
mod stream;
mod tool_parse;

pub use config::AgentConfig;
pub use error::AgentError;
pub use loop_impl::{AgentLoop, AgentTurnRequest, AgentTurnResult};
pub use stream::{AgentStreamEvent, AgentStreamSink};