//! Erreurs harness (onboard, doctor, daemon OS).

use std::path::PathBuf;

use thiserror::Error;

use crate::config::ConfigError;
use crate::OrchestratorError;

/// Erreur des opérations harness.
#[derive(Debug, Error)]
pub enum HarnessError {
    /// Configuration workspace illisible.
    #[error("config: {0}")]
    Config(#[from] ConfigError),
    /// Erreur orchestrateur (bridge, persistance).
    #[error("{0}")]
    Orchestrator(#[from] OrchestratorError),
    /// IO disque.
    #[error("io {path}: {message}")]
    Io {
        /// Chemin concerné.
        path: PathBuf,
        /// Détail.
        message: String,
    },
    /// Opération OS (schtasks, powershell).
    #[error("plateforme: {0}")]
    Platform(String),
    /// Diagnostic : une ou plusieurs vérifications ont échoué.
    #[error("doctor: {count} problème(s) détecté(s)")]
    DoctorFailed {
        /// Nombre de problèmes.
        count: usize,
    },
    /// Smoke harness interrompu.
    #[error("harness smoke échoué à l'étape: {step}")]
    SmokeFailed {
        /// Nom de l'étape.
        step: String,
    },
    /// Provider injoignable.
    #[error("probe provider: {0}")]
    ProviderProbe(String),
    /// Gateway injoignable pendant smoke.
    #[error("gateway injoignable ({url})")]
    GatewayDown {
        /// URL sonde.
        url: String,
    },
    /// Config absente.
    #[error("config absente — lancez onboard ({path})")]
    MissingConfig {
        /// Chemin attendu.
        path: PathBuf,
    },
}

impl HarnessError {
    /// Erreur IO avec chemin.
    pub fn io(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::Io {
            path: path.into(),
            message: message.into(),
        }
    }
}