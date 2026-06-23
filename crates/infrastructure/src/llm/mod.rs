//! Adapters du port [`orchestrator::LlmProvider`].
//!
//! Le trait [`LlmProvider`] vit dans `orchestrator::llm` (pas ici) pour éviter
//! une dépendance circulaire. Ce module fournit les implémentations concrètes :
//!
//! - [`XaiGrokProvider`] — xAI Grok (chat + structured output `MemoryDraft`)
//! - [`OllamaLlmProvider`] — Ollama local
//! - [`ChainedLlmProvider`] — fallback primary → fallbacks TOML

mod anthropic;
mod chain;
mod factory;
mod ollama_llm;
mod openai_compatible;
mod xai_grok;

pub use anthropic::AnthropicLlmProvider;
pub use chain::ChainedLlmProvider;
pub use factory::{build_llm_provider, LlmFactoryError};
pub use ollama_llm::OllamaLlmProvider;
pub use openai_compatible::OpenAiCompatibleLlmProvider;
pub use xai_grok::XaiGrokProvider;
