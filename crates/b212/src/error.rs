use thiserror::Error;

/// Erreurs du protocole B212.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum B212Error {
    /// Données marché indisponibles.
    #[error("market data: {0}")]
    MarketData(String),
    /// Fixture ou fichier introuvable.
    #[error("fixture not found: {path}")]
    FixtureNotFound {
        /// Chemin attendu.
        path: String,
    },
    /// JSON invalide.
    #[error("parse error: {0}")]
    Parse(String),
    /// Série OHLCV vide ou insuffisante.
    #[error("insufficient bars: need {need}, got {got}")]
    InsufficientBars {
        /// Minimum requis.
        need: usize,
        /// Reçu.
        got: usize,
    },
    /// Configuration B212 invalide.
    #[error("config: {0}")]
    Config(String),
    /// Règles cardinales violées — proposition interdite.
    #[error("cardinal rules: {0}")]
    CardinalRules(String),
    /// Proposition bloquée (scores ou gouvernance).
    #[error("proposal blocked: {0}")]
    ProposalBlocked(String),
    /// Proposition introuvable.
    #[error("proposal not found: {0}")]
    ProposalNotFound(String),
    /// Transition HITL invalide.
    #[error("invalid proposal status: {0}")]
    InvalidProposalStatus(String),
    /// Erreur journal ou persistance.
    #[error("journal: {0}")]
    Journal(String),
}