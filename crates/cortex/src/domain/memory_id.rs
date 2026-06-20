use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::CortexError;

/// Identifiant unique d'une mémoire (UUID v7 — tri temporel naturel).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MemoryId(Uuid);

impl MemoryId {
    /// Génère un nouvel identifiant UUID v7.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Construit depuis un UUID existant (doit être v7 ou valide).
    #[must_use]
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Accès à l'UUID sous-jacent.
    #[must_use]
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Représentation canonique string.
    #[must_use]
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl Default for MemoryId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for MemoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for MemoryId {
    type Err = CortexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s)
            .map(Self)
            .map_err(|e| CortexError::InvalidMemoryId(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_generates_valid_uuid_v7() {
        let id = MemoryId::new();
        assert_eq!(id.as_uuid().get_version(), Some(uuid::Version::SortRand));
    }

    #[test]
    fn roundtrip_from_str() {
        let id = MemoryId::new();
        let parsed: MemoryId = id.to_string().parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn invalid_str_returns_error() {
        let err = "not-a-uuid".parse::<MemoryId>().unwrap_err();
        assert!(matches!(err, CortexError::InvalidMemoryId(_)));
    }

    #[test]
    fn ordering_follows_uuid_order() {
        let a = MemoryId::new();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = MemoryId::new();
        assert!(a < b);
    }
}
