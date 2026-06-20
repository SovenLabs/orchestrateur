//! Adapters du port [`orchestrator::LlmProvider`].

mod chain;
mod factory;
mod ollama_llm;
mod xai_grok;

pub use chain::ChainedLlmProvider;
pub use factory::{build_llm_provider, LlmFactoryError};
pub use ollama_llm::OllamaLlmProvider;
pub use xai_grok::XaiGrokProvider;
