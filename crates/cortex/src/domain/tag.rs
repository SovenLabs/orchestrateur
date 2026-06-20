use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::CortexError;

/// Longueur maximale d'un tag (contrat domaine).
pub const TAG_MAX_LEN: usize = 64;

/// Tag normalisé (lowercase, sans espaces, max 64 caractères).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Tag(String);

impl Tag {
    /// Crée un tag normalisé.
    ///
    /// # Errors
    ///
    /// Retourne [`CortexError::InvalidTag`] ou [`CortexError::TagTooLong`] si le tag est invalide.
    pub fn new(raw: impl AsRef<str>) -> Result<Self, CortexError> {
        let normalized = raw.as_ref().trim().to_lowercase();
        if normalized.is_empty() {
            return Err(CortexError::InvalidTag("tag vide".into()));
        }
        if normalized.chars().any(char::is_whitespace) {
            return Err(CortexError::InvalidTag(
                "les tags ne peuvent pas contenir d'espaces".into(),
            ));
        }
        if normalized.len() > TAG_MAX_LEN {
            return Err(CortexError::TagTooLong);
        }
        Ok(Self(normalized))
    }

    /// Retourne la représentation normalisée du tag.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Tag {
    type Err = CortexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_to_lowercase() {
        let tag = Tag::new("Architecture").unwrap();
        assert_eq!(tag.as_str(), "architecture");
    }

    #[test]
    fn rejects_empty() {
        assert!(matches!(Tag::new("  "), Err(CortexError::InvalidTag(_))));
    }

    #[test]
    fn rejects_whitespace_inside() {
        assert!(matches!(
            Tag::new("bad tag"),
            Err(CortexError::InvalidTag(_))
        ));
    }

    #[test]
    fn rejects_too_long() {
        let long = "a".repeat(65);
        assert!(matches!(Tag::new(long), Err(CortexError::TagTooLong)));
    }
}
