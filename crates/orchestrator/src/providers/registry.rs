use super::descriptor::{ProviderDescriptor, ProviderKind, EMBEDDING_DESCRIPTORS, LLM_DESCRIPTORS};

/// Registre typé des providers supportés (Phase 9).
#[derive(Debug, Clone, Copy, Default)]
pub struct ProviderRegistry;

impl ProviderRegistry {
    /// Nouveau registre (catalogue statique).
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Tous les descripteurs LLM enregistrés.
    #[must_use]
    pub fn llm_descriptors(&self) -> &'static [ProviderDescriptor] {
        LLM_DESCRIPTORS
    }

    /// Tous les descripteurs embeddings enregistrés.
    #[must_use]
    pub fn embedding_descriptors(&self) -> &'static [ProviderDescriptor] {
        EMBEDDING_DESCRIPTORS
    }

    /// Recherche un descripteur LLM par identifiant.
    #[must_use]
    pub fn llm_descriptor(&self, id: &str) -> Option<&'static ProviderDescriptor> {
        Self::find(LLM_DESCRIPTORS, id)
    }

    /// Recherche un descripteur embedding par identifiant.
    #[must_use]
    pub fn embedding_descriptor(&self, id: &str) -> Option<&'static ProviderDescriptor> {
        Self::find(EMBEDDING_DESCRIPTORS, id)
    }

    /// Nombre total de providers enregistrés (LLM + embeddings).
    #[must_use]
    pub fn total_count(&self) -> usize {
        LLM_DESCRIPTORS.len() + EMBEDDING_DESCRIPTORS.len()
    }

    fn find(list: &'static [ProviderDescriptor], id: &str) -> Option<&'static ProviderDescriptor> {
        list.iter().find(|d| d.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_at_least_ten_llm_providers() {
        let registry = ProviderRegistry::new();
        assert!(registry.llm_descriptors().len() >= 10);
    }

    #[test]
    fn registry_resolves_openai() {
        let registry = ProviderRegistry::new();
        let d = registry.llm_descriptor("openai").expect("openai");
        assert_eq!(d.display_name, "OpenAI");
        assert_eq!(d.kind, ProviderKind::Llm);
    }
}