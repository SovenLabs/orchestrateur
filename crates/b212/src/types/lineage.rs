use serde::{Deserialize, Serialize};

use crate::version::B212_VERSION;

/// Traçabilité audit (source données + version protocole).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct B212Lineage {
    /// Source des données (`fixture`, `ccxt`, …).
    pub data_source: String,
    /// Chargeur / adapter utilisé.
    pub loader: String,
    /// Version Bible B212.
    pub b212_version: String,
    /// Horodatage analyse ISO-8601.
    pub analyzed_at: String,
}

impl B212Lineage {
    /// Crée une lignée pour fixtures workspace.
    #[must_use]
    pub fn fixture(loader: impl Into<String>) -> Self {
        Self {
            data_source: "fixture".into(),
            loader: loader.into(),
            b212_version: B212_VERSION.into(),
            analyzed_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}