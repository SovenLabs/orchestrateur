use serde::{Deserialize, Serialize};

use super::{CortexError, MemoryId};

/// Type de lien entre mémoires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BacklinkKind {
    Semantic,
    ExplicitWikilink,
}

/// Lien dirigé vers une autre mémoire, avec score de pertinence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Backlink {
    pub target: MemoryId,
    pub score: f32,
    pub kind: BacklinkKind,
}

impl Backlink {
    /// Construit un backlink validé (score ∈ [0.0, 1.0]).
    pub fn new(target: MemoryId, score: f32, kind: BacklinkKind) -> Result<Self, CortexError> {
        if !(0.0..=1.0).contains(&score) {
            return Err(CortexError::InvalidBacklink(format!(
                "score hors bornes: {score}"
            )));
        }
        Ok(Self { target, score, kind })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_score() {
        let bl = Backlink::new(MemoryId::new(), 0.87, BacklinkKind::Semantic).unwrap();
        assert!((bl.score - 0.87).abs() < f32::EPSILON);
    }

    #[test]
    fn rejects_score_above_one() {
        assert!(Backlink::new(MemoryId::new(), 1.1, BacklinkKind::Semantic).is_err());
    }

    #[test]
    fn serde_roundtrip() {
        let bl = Backlink::new(MemoryId::new(), 0.5, BacklinkKind::ExplicitWikilink).unwrap();
        let yaml = serde_yaml::to_string(&bl).unwrap();
        let parsed: Backlink = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(bl, parsed);
    }
}