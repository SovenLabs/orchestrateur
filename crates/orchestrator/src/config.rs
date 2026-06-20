use cortex::SimilarityThresholds;

/// Configuration applicative de l'orchestrateur (valeurs par défaut saines).
#[derive(Debug, Clone, PartialEq)]
pub struct OrchestratorConfig {
    /// Seuils pour le calcul des backlinks sémantiques.
    pub similarity_thresholds: SimilarityThresholds,
    /// Dimension des embeddings (utilisée par les mocks ; ignorée par les adapters réels).
    pub embedding_dim: usize,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            similarity_thresholds: SimilarityThresholds::default(),
            embedding_dim: 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_sane_thresholds() {
        let cfg = OrchestratorConfig::default();
        assert!((cfg.similarity_thresholds.semantic_min - 0.75).abs() < f32::EPSILON);
        assert_eq!(cfg.similarity_thresholds.max_links, 10);
        assert_eq!(cfg.embedding_dim, 8);
    }
}