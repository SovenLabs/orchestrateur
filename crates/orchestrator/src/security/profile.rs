//! Presets de sécurité — une ligne TOML pour adapter le profil d'usage.

use crate::config::{BehavioralConfig, SecurityConfig};

/// Profil de sécurité prédéfini.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityProfile {
    /// Valeurs conservatrices (défaut crate).
    Default,
    /// Sessions IA intensives — plafonds hauts, validation et audit actifs.
    AiAssisted,
    /// Posture renforcée — limites basses.
    Strict,
    /// Opérateur souverain — rate limiting et détection injection désactivés.
    Expert,
    /// Zéro egress cloud — LLM/embeddings locaux uniquement + scan secrets.
    LocalOnly,
}

impl SecurityProfile {
    /// Parse un nom de profil TOML (`ai_assisted`, `ai-assisted`, …).
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name.trim().to_lowercase().as_str() {
            "default" => Some(Self::Default),
            "ai_assisted" | "ai-assisted" | "ai" => Some(Self::AiAssisted),
            "strict" => Some(Self::Strict),
            "expert" => Some(Self::Expert),
            "local_only" | "local-only" | "local" => Some(Self::LocalOnly),
            _ => None,
        }
    }

    /// Applique le preset sur la configuration sécurité.
    ///
    /// Les champs TOML explicites fusionnés après l'appel prennent le pas.
    pub fn apply(self, config: &mut SecurityConfig) {
        config.profile = Some(self);
        match self {
            Self::Default => {
                *config = SecurityConfig::default();
                config.profile = Some(Self::Default);
            }
            Self::AiAssisted => apply_ai_assisted(config),
            Self::Strict => apply_strict(config),
            Self::Expert => apply_expert(config),
            Self::LocalOnly => apply_local_only(config),
        }
    }
}

fn apply_ai_assisted(config: &mut SecurityConfig) {
    config.enabled = true;
    config.detect_injection_patterns = true;
    config.behavioral = BehavioralConfig {
        enabled: true,
        max_assimilations_per_minute: 300,
        max_searches_per_minute: 600,
        max_repetitive_searches: 80,
        window_secs: 60,
        anomaly_block_threshold: 95.0,
    };
    config.integrity.enabled = true;
    config.integrity.verify_config_hash = true;
    config.integrity.bootstrap_on_missing = true;
    config.audit.enabled = true;
}

fn apply_strict(config: &mut SecurityConfig) {
    config.enabled = true;
    config.detect_injection_patterns = true;
    config.behavioral = BehavioralConfig {
        enabled: true,
        max_assimilations_per_minute: 20,
        max_searches_per_minute: 40,
        max_repetitive_searches: 5,
        window_secs: 60,
        anomaly_block_threshold: 60.0,
    };
    config.integrity.enabled = true;
    config.integrity.seed_honeypots = true;
    config.audit.enabled = true;
}

fn apply_expert(config: &mut SecurityConfig) {
    config.enabled = true;
    config.detect_injection_patterns = false;
    config.behavioral.enabled = false;
    config.integrity.enabled = true;
    config.audit.enabled = true;
}

fn apply_local_only(config: &mut SecurityConfig) {
    apply_strict(config);
    config.block_cloud_llm = true;
    config.scan_secrets_before_llm = true;
    config.integrity.seed_honeypots = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "sécurité: profil ai-assisted"]
    fn ai_assisted_raises_behavioral_limits() {
        let mut cfg = SecurityConfig::default();
        SecurityProfile::AiAssisted.apply(&mut cfg);
        assert_eq!(cfg.behavioral.max_assimilations_per_minute, 300);
        assert_eq!(cfg.behavioral.max_repetitive_searches, 80);
        assert!(cfg.enabled);
        assert!(cfg.detect_injection_patterns);
    }

    #[test]
    #[ignore = "sécurité: profil expert désactive garde-fous"]
    fn expert_disables_behavioral_and_injection_detection() {
        let mut cfg = SecurityConfig::default();
        SecurityProfile::Expert.apply(&mut cfg);
        assert!(!cfg.behavioral.enabled);
        assert!(!cfg.detect_injection_patterns);
    }

    #[test]
    fn parses_aliases() {
        assert_eq!(
            SecurityProfile::parse("ai-assisted"),
            Some(SecurityProfile::AiAssisted)
        );
        assert_eq!(SecurityProfile::parse("unknown"), None);
    }
}
