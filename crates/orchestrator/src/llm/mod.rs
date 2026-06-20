//! Ports LLM — génération de [`MemoryDraft`] et chat (couche orchestrator uniquement).

mod llm_provider;

pub use llm_provider::{ChatMessage, LlmCapabilities, LlmError, LlmProvider, LlmUsageRecorded};
