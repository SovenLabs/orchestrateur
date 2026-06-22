//! Adapters du port [`cortex::EmbeddingProvider`].

mod chain;
mod factory;
mod ollama_provider;
mod openai_embeddings;

pub use chain::ChainedEmbeddingProvider;
pub use factory::{build_embedding_provider, EmbeddingFactoryError};
pub use ollama_provider::OllamaEmbeddingProvider;
pub use openai_embeddings::OpenAiEmbeddingsProvider;
