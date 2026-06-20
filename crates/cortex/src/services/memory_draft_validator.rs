//! Validation adversariale des [`MemoryDraft`] — **dernière barrière domaine** avant persistance.
//!
//! Placé dans `cortex` (et non `orchestrator`) pour que toute voie d'écriture
//! vers la mémoire durable doive respecter les mêmes règles invariantes,
//! indépendamment de la couche applicative.

use std::collections::HashSet;
use std::str::FromStr;

use thiserror::Error;

use crate::domain::{MemoryDraft, MemoryId, Tag};

/// Configuration pure du validateur (sans dépendance TOML / orchestrator).
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDraftValidatorConfig {
    /// Longueur minimale du contenu (caractères).
    pub min_content_length: usize,
    /// Longueur maximale du contenu.
    pub max_content_length: usize,
    /// Longueur maximale du titre.
    pub max_title_length: usize,
    /// Nombre maximal de tags.
    pub max_tags: usize,
    /// Nombre maximal de backlinks candidats.
    pub max_backlinks: usize,
    /// Détecte les patterns d'injection connus.
    pub detect_injection_patterns: bool,
    /// Score de risque au-delà duquel la validation bloque (0.0–1.0).
    pub blocking_risk_threshold: f32,
}

impl Default for MemoryDraftValidatorConfig {
    fn default() -> Self {
        Self {
            min_content_length: 1,
            max_content_length: 64_000,
            max_title_length: 512,
            max_tags: 32,
            max_backlinks: 20,
            detect_injection_patterns: true,
            blocking_risk_threshold: 0.85,
        }
    }
}

/// Erreur de validation adversariale sur un [`MemoryDraft`].
#[derive(Debug, Clone, Error, PartialEq)]
pub enum ValidationError {
    /// Contenu trop court.
    #[error("contenu trop court ({actual} < {min} caractères)")]
    ContentTooShort {
        /// Minimum requis.
        min: usize,
        /// Longueur observée.
        actual: usize,
    },
    /// Contenu trop long.
    #[error("contenu trop long ({actual} > {max} caractères)")]
    ContentTooLong {
        /// Limite configurée.
        max: usize,
        /// Longueur observée.
        actual: usize,
    },
    /// Titre trop long.
    #[error("titre trop long ({actual} > {max} caractères)")]
    TitleTooLong {
        /// Limite configurée.
        max: usize,
        /// Longueur observée.
        actual: usize,
    },
    /// Trop de tags.
    #[error("trop de tags ({actual} > {max})")]
    TooManyTags {
        /// Limite configurée.
        max: usize,
        /// Nombre observé.
        actual: usize,
    },
    /// Trop de backlinks candidats.
    #[error("trop de backlinks ({actual} > {max})")]
    TooManyBacklinks {
        /// Maximal autorisé.
        max: usize,
        /// Nombre observé.
        actual: usize,
    },
    /// Tag hors allow-list.
    #[error("tag invalide: {0}")]
    InvalidTag(String),
    /// Cible de backlink non conforme.
    #[error("cible de backlink invalide: {0}")]
    InvalidBacklinkTarget(String),
    /// Score de backlink hors plage.
    #[error("score de backlink invalide: {score}")]
    InvalidBacklinkScore {
        /// Valeur observée.
        score: f32,
    },
    /// Backlink vers mémoire inexistante.
    #[error("backlink vers mémoire inexistante: {0}")]
    BacklinkTargetNotFound(String),
    /// Pattern d'injection ou de poisoning détecté.
    #[error("contenu suspect: {0}")]
    SuspiciousContent(String),
    /// Caractères de contrôle interdits.
    #[error("caractères de contrôle interdits détectés")]
    ControlCharacters,
    /// Répétition excessive (`DoS` / token stuffing).
    #[error("répétition excessive détectée")]
    ExcessiveRepetition,
}

/// Avertissement non bloquant (scoring gradué).
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationWarning {
    /// Code court (`injection_pattern`, `repeated_ngram`, …).
    pub code: String,
    /// Description lisible.
    pub message: String,
    /// Contribution au score de risque (0.0–1.0).
    pub risk_delta: f32,
}

/// Résultat détaillé de validation avec scoring de risque.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    /// Score agrégé de risque (0.0 = sain, 1.0 = critique).
    pub risk_score: f32,
    /// Indique si la persistance doit être refusée.
    pub is_blocking: bool,
    /// Première erreur bloquante (si présente).
    pub blocking_error: Option<ValidationError>,
    /// Avertissements non bloquants.
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Indique si le brouillon peut être persisté.
    #[must_use]
    pub fn is_acceptable(&self) -> bool {
        !self.is_blocking
    }
}

/// Validateur conservateur des brouillons — cœur de la défense adversariale du Cortex.
#[derive(Debug, Clone)]
pub struct MemoryDraftValidator {
    config: MemoryDraftValidatorConfig,
}

impl MemoryDraftValidator {
    /// Construit un validateur à partir d'une configuration domaine pure.
    #[must_use]
    pub fn from_config(config: &MemoryDraftValidatorConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Valide un brouillon avant conversion [`MemoryDraft::into_memory`].
    ///
    /// # Errors
    ///
    /// Retourne [`ValidationError`] si une règle de sécurité est violée.
    pub fn validate(&self, draft: &MemoryDraft) -> Result<(), ValidationError> {
        let detailed = self.validate_detailed(draft);
        if let Some(err) = detailed.blocking_error {
            return Err(err);
        }
        Ok(())
    }

    /// Valide avec scoring gradué et avertissements (sans bloquer sur seuil seul).
    #[must_use]
    pub fn validate_detailed(&self, draft: &MemoryDraft) -> ValidationResult {
        let mut extra_risk = 0.0_f32;

        let mut result = ValidationResult {
            risk_score: 0.0,
            is_blocking: false,
            blocking_error: None,
            warnings: Vec::new(),
        };

        let record_blocking = |err: ValidationError, delta: f32, result: &mut ValidationResult| {
            result.risk_score = (result.risk_score + delta).min(1.0);
            if result.blocking_error.is_none() {
                result.blocking_error = Some(err);
                result.is_blocking = true;
            }
        };

        if let Err(err) = self.check_lengths(draft) {
            record_blocking(err, 0.9, &mut result);
        }
        if let Err(err) = Self::check_tags(draft) {
            record_blocking(err, 0.7, &mut result);
        }
        if let Err(err) = Self::check_backlinks(draft) {
            record_blocking(err, 0.7, &mut result);
        }

        for field in [&draft.title, &draft.content] {
            if let Err(err) = Self::check_control_characters(field) {
                record_blocking(err, 0.95, &mut result);
            } else {
                match check_repetition(field) {
                    RepetitionCheck::Blocking => {
                        record_blocking(ValidationError::ExcessiveRepetition, 0.85, &mut result);
                    }
                    RepetitionCheck::Warning { code, message, delta } => {
                        extra_risk = (extra_risk + delta).min(1.0);
                        result.warnings.push(ValidationWarning {
                            code,
                            message,
                            risk_delta: delta,
                        });
                    }
                    RepetitionCheck::Clean => {}
                }

                if self.config.detect_injection_patterns {
                    if let Some(label) = find_injection_pattern(field) {
                        record_blocking(
                            ValidationError::SuspiciousContent(label.to_string()),
                            0.95,
                            &mut result,
                        );
                    }
                }
            }
        }

        result.risk_score = (result.risk_score + extra_risk).min(1.0);
        if !result.is_blocking && result.risk_score >= self.config.blocking_risk_threshold {
            result.is_blocking = true;
            if result.blocking_error.is_none() {
                result.blocking_error = Some(ValidationError::SuspiciousContent(
                    "score de risque au-dessus du seuil".into(),
                ));
            }
        }
        result
    }

    /// Valide le brouillon et vérifie que les cibles de backlink existent dans `known_ids`.
    ///
    /// # Errors
    ///
    /// Retourne [`ValidationError`] si une règle ou une cible de backlink est invalide.
    pub fn validate_with_known_ids(
        &self,
        draft: &MemoryDraft,
        known_ids: &HashSet<MemoryId>,
    ) -> Result<(), ValidationError> {
        self.validate(draft)?;
        for bl in &draft.backlinks {
            let id = MemoryId::from_str(&bl.target).map_err(|_| {
                ValidationError::InvalidBacklinkTarget(bl.target.clone())
            })?;
            if !known_ids.contains(&id) {
                return Err(ValidationError::BacklinkTargetNotFound(bl.target.clone()));
            }
        }
        Ok(())
    }

    fn check_lengths(&self, draft: &MemoryDraft) -> Result<(), ValidationError> {
        let trimmed = draft.content.trim();
        if trimmed.len() < self.config.min_content_length {
            return Err(ValidationError::ContentTooShort {
                min: self.config.min_content_length,
                actual: trimmed.len(),
            });
        }
        if draft.content.len() > self.config.max_content_length {
            return Err(ValidationError::ContentTooLong {
                max: self.config.max_content_length,
                actual: draft.content.len(),
            });
        }
        if draft.title.len() > self.config.max_title_length {
            return Err(ValidationError::TitleTooLong {
                max: self.config.max_title_length,
                actual: draft.title.len(),
            });
        }
        if draft.tags.len() > self.config.max_tags {
            return Err(ValidationError::TooManyTags {
                max: self.config.max_tags,
                actual: draft.tags.len(),
            });
        }
        if draft.backlinks.len() > self.config.max_backlinks {
            return Err(ValidationError::TooManyBacklinks {
                max: self.config.max_backlinks,
                actual: draft.backlinks.len(),
            });
        }
        Ok(())
    }

    fn check_tags(draft: &MemoryDraft) -> Result<(), ValidationError> {
        for tag in &draft.tags {
            let normalized = match Tag::new(tag) {
                Ok(t) => t.as_str().to_string(),
                Err(_) => return Err(ValidationError::InvalidTag(tag.clone())),
            };
            if !is_allowed_tag_chars(&normalized) {
                return Err(ValidationError::InvalidTag(tag.clone()));
            }
        }
        Ok(())
    }

    fn check_backlinks(draft: &MemoryDraft) -> Result<(), ValidationError> {
        for bl in &draft.backlinks {
            if MemoryId::from_str(&bl.target).is_err() {
                return Err(ValidationError::InvalidBacklinkTarget(bl.target.clone()));
            }
            if !(0.0..=1.0).contains(&bl.score) || bl.score.is_nan() {
                return Err(ValidationError::InvalidBacklinkScore { score: bl.score });
            }
        }
        Ok(())
    }

    fn check_control_characters(text: &str) -> Result<(), ValidationError> {
        if text.chars().any(is_forbidden_control_char) {
            return Err(ValidationError::ControlCharacters);
        }
        Ok(())
    }
}

/// Résultat interne de la détection de répétition.
enum RepetitionCheck {
    Clean,
    Warning {
        code: String,
        message: String,
        delta: f32,
    },
    Blocking,
}

/// Détecte répétitions de caractères et n-grammes excessifs.
fn check_repetition(text: &str) -> RepetitionCheck {
    const CHAR_RUN_BLOCK: usize = 80;
    const CHAR_RUN_WARN: usize = 40;
    const NGRAM_LEN: usize = 4;
    const NGRAM_REPEAT_BLOCK: usize = 8;
    const NGRAM_REPEAT_WARN: usize = 5;

    let mut run_char = None;
    let mut run_len = 0usize;
    for ch in text.chars() {
        if Some(ch) == run_char {
            run_len += 1;
            if run_len >= CHAR_RUN_BLOCK {
                return RepetitionCheck::Blocking;
            }
        } else {
            run_char = Some(ch);
            run_len = 1;
        }
    }
    if run_len >= CHAR_RUN_WARN {
        return RepetitionCheck::Warning {
            code: "char_run".into(),
            message: format!("répétition de '{run_char:?}' ({run_len} fois)"),
            delta: 0.25,
        };
    }

    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() >= NGRAM_LEN {
        let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for window in words.windows(NGRAM_LEN) {
            let key = window.join(" ");
            let count = counts.entry(key).or_insert(0);
            *count += 1;
            if *count >= NGRAM_REPEAT_BLOCK {
                return RepetitionCheck::Blocking;
            }
        }
        if let Some((phrase, count)) = counts.iter().max_by_key(|(_, c)| *c) {
            if *count >= NGRAM_REPEAT_WARN {
                return RepetitionCheck::Warning {
                    code: "repeated_ngram".into(),
                    message: format!("n-gramme répété {count} fois: \"{phrase}\""),
                    delta: 0.35,
                };
            }
        }
    }

    RepetitionCheck::Clean
}

fn is_forbidden_control_char(c: char) -> bool {
    c.is_control() && c != '\n' && c != '\r' && c != '\t'
}

fn is_allowed_tag_chars(tag: &str) -> bool {
    !tag.is_empty()
        && tag
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

fn normalize_for_scan(text: &str) -> String {
    text.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

const INJECTION_PHRASES: &[(&str, &str)] = &[
    ("ignore all previous instructions", "ignore all previous instructions"),
    ("ignore previous instructions", "ignore previous instructions"),
    ("disregard all prior", "disregard all prior"),
    ("disregard prior", "disregard prior"),
    ("system prompt", "system prompt"),
    ("you are now", "you are now"),
    ("do not follow", "do not follow"),
    ("override your instructions", "override your instructions"),
    ("override instructions", "override instructions"),
    ("dump all memories", "dump all memories"),
    ("dump all data", "dump all data"),
    ("dump all secrets", "dump all secrets"),
    ("dump memories", "dump memories"),
    ("dump secrets", "dump secrets"),
    ("reveal the api", "reveal the api"),
    ("reveal api", "reveal api"),
    ("reveal secret", "reveal secret"),
    ("reveal key", "reveal key"),
    ("reveal password", "reveal password"),
    ("jailbreak", "jailbreak"),
    ("developer mode", "developer mode"),
];

/// Détecte un pattern d'injection connu (titre ou contenu).
#[must_use]
pub fn find_injection_pattern(text: &str) -> Option<&'static str> {
    let normalized = normalize_for_scan(text);
    for (needle, label) in INJECTION_PHRASES {
        if normalized.contains(needle) {
            return Some(label);
        }
    }

    let lower = text.to_lowercase();
    if lower.contains("exfiltrat") {
        return Some("exfiltrat");
    }
    if lower.contains("<script") {
        return Some("<script");
    }
    if lower.contains("{{") && lower.contains("}}") {
        return Some("{{ }}");
    }
    if lower.contains("{%") && lower.contains("%}") {
        return Some("{% %}");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{BacklinkDraft, BacklinkDraftKind, MemoryDraft};

    fn validator() -> MemoryDraftValidator {
        MemoryDraftValidator::from_config(&MemoryDraftValidatorConfig::default())
    }

    fn valid_draft() -> MemoryDraft {
        MemoryDraft {
            title: "Décision valide".into(),
            content: "Contenu markdown sain.".into(),
            tags: vec!["architecture".into()],
            backlinks: vec![],
        }
    }

    #[test]
    fn accepts_valid_draft() {
        validator().validate(&valid_draft()).expect("draft valide");
    }

    #[test]
    fn rejects_content_too_long() {
        let mut draft = valid_draft();
        draft.content = "x".repeat(65_000);
        let err = validator().validate(&draft).unwrap_err();
        assert!(matches!(err, ValidationError::ContentTooLong { .. }));
    }

    #[test]
    fn rejects_invalid_tag_chars() {
        let mut draft = valid_draft();
        draft.tags = vec!["bad tag".into()];
        let err = validator().validate(&draft).unwrap_err();
        assert!(matches!(err, ValidationError::InvalidTag(_)));
    }

    #[test]
    fn rejects_too_many_backlinks() {
        let mut draft = valid_draft();
        draft.backlinks = (0..25)
            .map(|_| BacklinkDraft {
                target: MemoryId::new().to_string(),
                score: 0.5,
                kind: BacklinkDraftKind::Semantic,
            })
            .collect();
        let err = validator().validate(&draft).unwrap_err();
        assert!(matches!(err, ValidationError::TooManyBacklinks { .. }));
    }

    #[test]
    fn repetition_allows_natural_prose() {
        let mut draft = valid_draft();
        draft.content =
            "Le Cortex valide les brouillons avant persistance. Chaque mémoire est un fichier."
                .into();
        validator().validate(&draft).expect("prose normale");
    }

    #[test]
    fn rejects_invalid_backlink_target() {
        let mut draft = valid_draft();
        draft.backlinks = vec![BacklinkDraft {
            target: "not-a-uuid".into(),
            score: 0.5,
            kind: BacklinkDraftKind::Semantic,
        }];
        let err = validator().validate(&draft).unwrap_err();
        assert!(matches!(err, ValidationError::InvalidBacklinkTarget(_)));
    }

}