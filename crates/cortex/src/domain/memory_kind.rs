use serde::{Deserialize, Serialize};

/// Type sémantique d'un insight / souvenir (aligné Pulse).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind {
    /// Choix ou arbitrage documenté.
    Decision,
    /// Piste abandonnée ou échec utile.
    DeadEnd,
    /// Règle réutilisable ou anti-pattern.
    Pattern,
    /// Contexte, état du projet, background.
    #[default]
    Context,
    /// Avancement, jalon, statut.
    Progress,
    /// Impact produit, marché, utilisateur.
    Business,
}

impl MemoryKind {
    /// Libellé court pour l'UI.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Decision => "Décision",
            Self::DeadEnd => "Impasse",
            Self::Pattern => "Pattern",
            Self::Context => "Contexte",
            Self::Progress => "Progrès",
            Self::Business => "Business",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_context() {
        assert_eq!(MemoryKind::default(), MemoryKind::Context);
    }

    #[test]
    fn serde_roundtrip() {
        let kind = MemoryKind::Decision;
        let json = serde_json::to_string(&kind).unwrap();
        let parsed: MemoryKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, parsed);
    }
}