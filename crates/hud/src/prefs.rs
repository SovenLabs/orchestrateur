//! Préférences UI persistées localement via [`eframe::Storage`].

use serde::{Deserialize, Serialize};

use crate::state::{HudState, LeftPanelMode};

/// Clé de persistance eframe.
pub const STORAGE_KEY: &str = "orchestrateur_hud_prefs_v1";

/// Sous-ensemble de [`HudState`] sérialisable entre sessions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPreferences {
    /// Thème sombre actif.
    pub dark_mode: bool,
    /// Afficher les métriques frame.
    pub show_frame_metrics: bool,
    /// Panneau assimilation ouvert.
    pub show_assimilate: bool,
    /// Filtre liste mémorisé.
    pub list_filter: String,
    /// Dernière requête de recherche.
    pub search_query: String,
    /// Panneau gauche actif (`memories` ou `search`).
    pub left_panel: PersistedPanel,
}

/// Panneau gauche persisté.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersistedPanel {
    /// Liste complète.
    Memories,
    /// Résultats de recherche.
    Search,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            dark_mode: true,
            show_frame_metrics: true,
            show_assimilate: false,
            list_filter: String::new(),
            search_query: String::new(),
            left_panel: PersistedPanel::Memories,
        }
    }
}

impl UiPreferences {
    /// Extrait les préférences depuis l'état courant.
    #[must_use]
    pub fn from_state(state: &HudState) -> Self {
        Self {
            dark_mode: state.dark_mode,
            show_frame_metrics: state.show_frame_metrics,
            show_assimilate: state.show_assimilate,
            list_filter: state.list_filter.clone(),
            search_query: state.search_query.clone(),
            left_panel: match state.left_panel {
                LeftPanelMode::Memories => PersistedPanel::Memories,
                LeftPanelMode::SearchResults => PersistedPanel::Search,
            },
        }
    }

    /// Applique les préférences sur l'état HUD.
    pub fn apply_to(self, state: &mut HudState) {
        state.dark_mode = self.dark_mode;
        state.show_frame_metrics = self.show_frame_metrics;
        state.show_assimilate = self.show_assimilate;
        state.list_filter = self.list_filter;
        state.search_query = self.search_query;
        state.left_panel = match self.left_panel {
            PersistedPanel::Memories => LeftPanelMode::Memories,
            PersistedPanel::Search => LeftPanelMode::SearchResults,
        };
    }

    /// Charge depuis le stockage eframe si présent.
    pub fn load_from_storage(storage: &dyn eframe::Storage) -> Option<Self> {
        let json = storage.get_string(STORAGE_KEY)?;
        serde_json::from_str(&json).ok()
    }

    /// Sauvegarde dans le stockage eframe.
    pub fn save_to_storage(&self, storage: &mut dyn eframe::Storage) {
        if let Ok(json) = serde_json::to_string(self) {
            storage.set_string(STORAGE_KEY, json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_serde() {
        let prefs = UiPreferences {
            list_filter: "rust".into(),
            ..UiPreferences::default()
        };
        let json = serde_json::to_string(&prefs).unwrap();
        let decoded: UiPreferences = serde_json::from_str(&json).unwrap();
        assert_eq!(prefs, decoded);
    }
}