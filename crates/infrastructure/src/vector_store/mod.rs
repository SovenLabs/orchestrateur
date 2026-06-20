//! Adapters du port [`cortex::VectorStore`].

mod factory;
mod lancedb_store;

pub use factory::{build_vector_store, VectorStoreFactoryError};
pub use lancedb_store::LancedbVectorStore;
