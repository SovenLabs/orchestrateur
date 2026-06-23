//! Garde-fous egress LLM — scan secrets et blocage providers cloud (profil `local_only`).

use crate::config::{OrchestratorConfig, SecurityConfig};

/// Erreur de politique egress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EgressError {
    /// Code court (`cloud_blocked`, `secret_detected`, …).
    pub code: String,
    /// Message opérateur.
    pub message: String,
}

/// Détection d'un motif sensible dans un texte utilisateur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretFinding {
    /// Libellé du motif.
    pub label: String,
}

const LOCAL_LLM_IDS: &[&str] = &["ollama", "lmstudio", "llamacpp", "local"];

fn looks_like_secret_token(slice: &str) -> bool {
    slice.len() >= 20
        && slice
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Providers LLM considérés locaux (pas d'egress cloud).
#[must_use]
pub fn is_local_llm_provider(id: &str) -> bool {
    let id = id.trim().to_lowercase();
    LOCAL_LLM_IDS
        .iter()
        .any(|local| id == *local || id.contains(local))
}

/// Scan basique des secrets dans un texte avant appel LLM.
#[must_use]
pub fn scan_secrets(text: &str) -> Vec<SecretFinding> {
    let lower = text.to_lowercase();
    let mut findings = Vec::new();
    if lower.contains("-----begin") && lower.contains("private key") {
        findings.push(SecretFinding {
            label: "private_key".into(),
        });
    }
    if lower.contains("bearer ") {
        findings.push(SecretFinding {
            label: "bearer".into(),
        });
    }
    for (label, prefix) in [
        ("openai_sk", "sk-"),
        ("aws_key", "akia"),
        ("xai_key", "xai-"),
        ("github_pat", "ghp_"),
    ] {
        if let Some(idx) = lower.find(prefix) {
            let tail = &text[idx + prefix.len()..];
            if looks_like_secret_token(tail.split_whitespace().next().unwrap_or("")) {
                findings.push(SecretFinding {
                    label: label.into(),
                });
            }
        }
    }
    findings
}

/// Vérifie que l'appel LLM est autorisé par la politique egress.
pub fn assert_llm_egress_allowed(config: &OrchestratorConfig) -> Result<(), EgressError> {
    let security = &config.security;
    if !security.block_cloud_llm {
        return Ok(());
    }
    let primary = config.providers.primary_llm.trim().to_lowercase();
    if !is_local_llm_provider(&primary) {
        return Err(EgressError {
            code: "cloud_blocked".into(),
            message: format!(
                "profil egress : provider LLM '{primary}' bloqué — utilisez ollama/lmstudio ou désactivez block_cloud_llm"
            ),
        });
    }
    for fallback in &config.providers.fallback_llm {
        if !is_local_llm_provider(fallback) {
            return Err(EgressError {
                code: "cloud_blocked".into(),
                message: format!(
                    "profil egress : fallback LLM '{fallback}' bloqué en mode local_only"
                ),
            });
        }
    }
    Ok(())
}

/// Vérifie le texte utilisateur avant envoi au LLM.
pub fn assert_text_safe_for_llm(
    security: &SecurityConfig,
    text: &str,
) -> Result<(), EgressError> {
    if !security.scan_secrets_before_llm {
        return Ok(());
    }
    let findings = scan_secrets(text);
    if findings.is_empty() {
        return Ok(());
    }
    let labels: Vec<_> = findings.iter().map(|f| f.label.as_str()).collect();
    Err(EgressError {
        code: "secret_detected".into(),
        message: format!(
            "contenu bloqué avant appel LLM — motifs sensibles détectés : {}",
            labels.join(", ")
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_openai_key_pattern() {
        let hits = scan_secrets("ma clé sk-abcdefghijklmnopqrstuvwxyz123456");
        assert!(!hits.is_empty());
    }

    #[test]
    fn local_only_blocks_xai_primary() {
        let mut config = OrchestratorConfig::default();
        config.security.block_cloud_llm = true;
        config.providers.primary_llm = "xai".into();
        assert!(assert_llm_egress_allowed(&config).is_err());
    }

    #[test]
    fn ollama_allowed_when_local_only() {
        let mut config = OrchestratorConfig::default();
        config.security.block_cloud_llm = true;
        config.providers.primary_llm = "ollama".into();
        assert!(assert_llm_egress_allowed(&config).is_ok());
    }
}