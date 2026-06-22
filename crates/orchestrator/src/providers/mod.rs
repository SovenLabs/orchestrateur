//! Registre typé des providers LLM et embeddings (Phase 9).

mod descriptor;
mod profile;
mod registry;

pub use descriptor::{ApiFamily, ProviderDescriptor, ProviderKind, EMBEDDING_DESCRIPTORS, LLM_DESCRIPTORS};
pub use profile::{ProviderProfile, ProviderProfileSection, ProviderProfiles};
pub use registry::ProviderRegistry;