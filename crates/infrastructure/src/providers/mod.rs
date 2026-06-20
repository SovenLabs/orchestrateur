//! Providers de repli lorsque la résolution initiale échoue.

mod unavailable;

pub use unavailable::{UnavailableEmbeddingProvider, UnavailableLlmProvider};
