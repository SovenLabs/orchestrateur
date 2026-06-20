//! Démarrage partagé HUD / CLI / TUI — chargement config + wiring.

use std::path::Path;

use orchestrator::{AppDependencies, ConfigError, OrchestratorConfig};
use thiserror::Error;

use crate::wiring::{build_app_dependencies, WiringError};

/// Message utilisateur lorsque `vector_store.type = memory` (réservé aux tests).
pub const MEMORY_MODE_HINT: &str =
    "vector_store type=memory : configurez type=lancedb dans orchestrator.toml";

/// Erreur de bootstrap applicatif.
#[derive(Debug, Error)]
pub enum BootstrapError {
    /// Chargement TOML / workspace.
    #[error(transparent)]
    Config(#[from] ConfigError),
    /// Composition des adapters.
    #[error(transparent)]
    Wiring(#[from] WiringError),
}

impl BootstrapError {
    /// Indique un refus explicite du mode `memory` hors tests.
    #[must_use]
    pub fn is_memory_mode(&self) -> bool {
        matches!(self, Self::Wiring(WiringError::MemoryMode))
    }

    /// Message lisible incluant le contexte binaire (`HUD`, `CLI`, `TUI`).
    #[must_use]
    pub fn with_context(self, app: &str) -> String {
        if self.is_memory_mode() {
            format!("{MEMORY_MODE_HINT} pour {app}")
        } else {
            self.to_string()
        }
    }
}

/// Charge la configuration workspace et construit [`AppDependencies`] production.
///
/// # Errors
///
/// Retourne [`BootstrapError`] si la config ou un adapter échoue.
pub async fn bootstrap_workspace(workspace: &Path) -> Result<AppDependencies, BootstrapError> {
    let config = OrchestratorConfig::load_workspace(workspace)?;
    build_app_dependencies(config).await.map_err(BootstrapError::from)
}