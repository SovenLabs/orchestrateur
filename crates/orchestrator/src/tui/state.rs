//! État du TUI — testable, sans logique domaine.

use crate::{format_health_status, MemoryDetailView, MemorySummary, Response};

/// Vue active du TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Liste des mémoires (navigation vim-like).
    List,
    /// Détail d'une mémoire.
    Detail,
    /// Saisie d'assimilation.
    Assimilate,
    /// Aide modale.
    Help,
}

/// État mutable de l'application TUI.
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct AppState {
    /// Catalogue complet issu du bridge (liste ou recherche).
    pub catalog: Vec<MemorySummary>,
    /// Vue filtrée affichée (sous-ensemble de `catalog`).
    pub items: Vec<MemorySummary>,
    /// Total rapporté par le bridge.
    pub total: usize,
    /// Index sélectionné dans `items`.
    pub selected_index: usize,
    /// Vue courante.
    pub current_view: View,
    /// Filtre local ou requête de recherche en cours de saisie.
    pub input_buffer: String,
    /// Mode recherche sémantique (`/` → Enter envoie `Command::Search`).
    pub search_mode: bool,
    /// Détail affiché.
    pub detail: Option<MemoryDetailView>,
    /// Texte d'assimilation en cours.
    pub assimilate_text: String,
    /// Message barre de statut.
    pub status_message: String,
    /// Version orchestrateur (health check).
    pub version: Option<String>,
    /// Demande de sortie.
    pub should_quit: bool,
    /// Affiche des résultats de recherche vectorielle.
    pub showing_search_results: bool,
    /// Provider LLM joignable.
    pub llm_available: bool,
    /// Provider embeddings joignable.
    pub embedding_available: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            catalog: Vec::new(),
            items: Vec::new(),
            total: 0,
            selected_index: 0,
            current_view: View::List,
            input_buffer: String::new(),
            search_mode: false,
            detail: None,
            assimilate_text: String::new(),
            status_message: "Prêt".to_string(),
            version: None,
            should_quit: false,
            showing_search_results: false,
            llm_available: true,
            embedding_available: true,
        }
    }
}

impl AppState {
    /// Applique une [`Response`] bridge.
    pub fn apply_response(&mut self, response: Response) {
        match response {
            Response::Health {
                status,
                version,
                llm_available,
                embedding_available,
            } => {
                self.version = Some(version);
                self.llm_available = llm_available;
                self.embedding_available = embedding_available;
                self.status_message =
                    format_health_status(&status, llm_available, embedding_available);
            }
            Response::MemoryList { items, total } => {
                self.catalog = items;
                self.total = total;
                self.showing_search_results = false;
                self.apply_local_filter();
                self.status_message = format!("{total} mémoire(s)");
            }
            Response::MemoryDetail { memory } => {
                self.detail = Some(MemoryDetailView::from_memory(&memory));
                self.current_view = View::Detail;
                self.status_message = "Détail chargé".to_string();
            }
            Response::SearchResults { items } => {
                self.catalog = items
                    .iter()
                    .map(|hit| MemorySummary {
                        id: hit.memory_id,
                        title: hit
                            .snippet
                            .clone()
                            .unwrap_or_else(|| hit.memory_id.to_string()),
                        tags: Vec::new(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        backlink_count: 0,
                    })
                    .collect();
                self.total = self.catalog.len();
                self.showing_search_results = true;
                self.apply_local_filter();
                self.status_message = format!("{} résultat(s)", self.total);
            }
            Response::Error(err) => {
                self.status_message = format!("[{}] {}", err.kind, err.message);
            }
            Response::Success { message } => {
                self.status_message = message;
            }
            Response::Event(_) => {}
        }
    }

    /// Filtre local instantané (titre/tags) sur le catalogue courant.
    pub fn apply_local_filter(&mut self) {
        if self.input_buffer.is_empty() {
            self.items = self.catalog.clone();
        } else {
            let needle = self.input_buffer.to_lowercase();
            self.items = self
                .catalog
                .iter()
                .filter(|item| {
                    item.title.to_lowercase().contains(&needle)
                        || item
                            .tags
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(&needle))
                })
                .cloned()
                .collect();
        }
        self.clamp_selection();
    }

    /// Sélection suivante.
    pub fn select_next(&mut self) {
        if !self.items.is_empty() && self.selected_index + 1 < self.items.len() {
            self.selected_index += 1;
        }
    }

    /// Sélection précédente.
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Mémoire sélectionnée.
    #[must_use]
    pub fn selected(&self) -> Option<&MemorySummary> {
        self.items.get(self.selected_index)
    }

    fn clamp_selection(&mut self) {
        if self.items.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.items.len() {
            self.selected_index = self.items.len() - 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(title: &str) -> MemorySummary {
        MemorySummary {
            id: cortex::MemoryId::new(),
            title: title.into(),
            tags: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            backlink_count: 0,
        }
    }

    #[test]
    fn select_next_and_previous() {
        let mut state = AppState::default();
        state.catalog = vec![sample("A"), sample("B")];
        state.items = state.catalog.clone();
        state.select_next();
        assert_eq!(state.selected_index, 1);
        state.select_previous();
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn local_filter_narrows_items() {
        let mut state = AppState::default();
        state.catalog = vec![sample("Rust"), sample("Python")];
        state.input_buffer = "rust".into();
        state.apply_local_filter();
        assert_eq!(state.items.len(), 1);
        assert_eq!(state.items[0].title, "Rust");
    }
}
