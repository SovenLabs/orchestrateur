mod agent_ports;
mod embedding_provider;
mod memory_repository;
mod session_repository;
mod vector_store;

pub use agent_ports::{
    AgentContext, AssimilationError, AssimilationPolicy, AssimilationResult,
    AssimilationService, ContextProvider, ContextSearchHit, RetrievalError, SemanticSearch,
};
pub use embedding_provider::{Embedding, EmbeddingCapabilities, EmbeddingError, EmbeddingProvider};
pub use memory_repository::MemoryRepository;
pub use session_repository::SessionRepository;
pub use vector_store::{SearchFilter, SearchHit, VectorStore};
