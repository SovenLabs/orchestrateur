//! État du TUI — testable, sans logique domaine.

use crate::{
    format_health_status, AuditEvent, DomainEvent, HubSummary, MemoryDetailView, MemorySummary,
    Response,
};

/// Vue active du TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Liste des mémoires (navigation vim-like).
    List,
    /// Détail d'une mémoire.
    Detail,
    /// Saisie d'assimilation.
    Assimilate,
    /// Graphe de connaissances.
    Graph,
    /// Journal d'audit.
    Audit,
    /// Chat libre LLM.
    Chat,
    /// Aide modale.
    Help,
}

/// Action suggérée après application d'une réponse ou d'un événement domaine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TuiAction {
    /// Aucune action supplémentaire.
    #[default]
    None,
    /// Recharger la liste des mémoires.
    RefreshList,
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
    /// Index sélectionné dans `items` ou dans `graph_hubs`.
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
    /// Nombre de nœuds du graphe.
    pub graph_node_count: usize,
    /// Nombre d'arêtes du graphe.
    pub graph_edge_count: usize,
    /// Hubs du graphe (backlinks entrants).
    pub graph_hubs: Vec<HubSummary>,
    /// Entrées d'audit récentes.
    pub audit_entries: Vec<AuditEvent>,
    /// Chaîne d'audit BLAKE3 intacte.
    pub audit_chain_intact: bool,
    /// Saisie chat en cours.
    pub chat_input: String,
    /// Dernière réponse LLM.
    pub chat_reply: Option<String>,
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
            graph_node_count: 0,
            graph_edge_count: 0,
            graph_hubs: Vec::new(),
            audit_entries: Vec::new(),
            audit_chain_intact: true,
            chat_input: String::new(),
            chat_reply: None,
        }
    }
}

impl AppState {
    /// Applique une [`Response`] bridge.
    #[must_use]
    pub fn apply_response(&mut self, response: Response) -> TuiAction {
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
                TuiAction::None
            }
            Response::MemoryList { items, total } => {
                self.catalog = items;
                self.total = total;
                self.showing_search_results = false;
                self.apply_local_filter();
                self.status_message = format!("{total} mémoire(s)");
                TuiAction::None
            }
            Response::MemoryDetail { memory } => {
                self.detail = Some(MemoryDetailView::from_memory(&memory));
                self.current_view = View::Detail;
                self.status_message = "Détail chargé".to_string();
                TuiAction::None
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
                TuiAction::None
            }
            Response::Error(err) => {
                self.status_message = format!("[{}] {}", err.kind, err.message);
                TuiAction::None
            }
            Response::Success { message } | Response::SkillResult { message } => {
                self.status_message = message;
                TuiAction::None
            }
            Response::Assimilated { memory_id, title } => {
                self.status_message = format!("Assimilé : {title} ({memory_id})");
                TuiAction::RefreshList
            }
            Response::GraphSummary {
                node_count,
                edge_count,
                hubs,
            } => {
                self.graph_node_count = node_count;
                self.graph_edge_count = edge_count;
                self.graph_hubs = hubs;
                self.clamp_graph_selection();
                self.status_message =
                    format!("Graphe : {node_count} nœuds, {edge_count} arêtes");
                TuiAction::None
            }
            Response::AuditLog {
                entries,
                chain_intact,
            } => {
                self.audit_entries = entries;
                self.audit_chain_intact = chain_intact;
                let status = if chain_intact { "intacte" } else { "ROMPUE" };
                self.status_message =
                    format!("Audit : {} entrée(s), chaîne {status}", self.audit_entries.len());
                TuiAction::None
            }
            Response::ChatReply { reply } => {
                self.chat_reply = Some(reply);
                self.status_message = "Réponse chat reçue".to_string();
                TuiAction::None
            }
            Response::SkillList { skills } => {
                self.status_message = format!("{} skill(s) disponibles", skills.len());
                TuiAction::None
            }
            Response::Event(event) => self.apply_domain_event(event),
        }
    }

    /// Applique un événement domaine poussé par le fan-out.
    #[must_use]
    pub fn apply_domain_event(&mut self, event: DomainEvent) -> TuiAction {
        match event {
            DomainEvent::MemoryAssimilated(payload) => {
                self.status_message =
                    format!("Événement : assimilation {}", payload.memory_id);
                TuiAction::RefreshList
            }
            DomainEvent::KnowledgeGraphValidated(payload) => {
                self.status_message = format!(
                    "Graphe validé — {} nœuds, {} arêtes",
                    payload.node_count, payload.edge_count
                );
                TuiAction::None
            }
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

    /// Sélection suivante (liste ou hubs graphe).
    pub fn select_next(&mut self) {
        let len = self.selection_len();
        if len > 0 && self.selected_index + 1 < len {
            self.selected_index += 1;
        }
    }

    /// Sélection précédente (liste ou hubs graphe).
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Mémoire sélectionnée dans la liste.
    #[must_use]
    pub fn selected(&self) -> Option<&MemorySummary> {
        self.items.get(self.selected_index)
    }

    /// Hub sélectionné dans la vue graphe.
    #[must_use]
    pub fn selected_hub(&self) -> Option<&HubSummary> {
        self.graph_hubs.get(self.selected_index)
    }

    fn selection_len(&self) -> usize {
        match self.current_view {
            View::Graph => self.graph_hubs.len(),
            _ => self.items.len(),
        }
    }

    fn clamp_selection(&mut self) {
        if self.items.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.items.len() {
            self.selected_index = self.items.len() - 1;
        }
    }

    fn clamp_graph_selection(&mut self) {
        if self.graph_hubs.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.graph_hubs.len() {
            self.selected_index = self.graph_hubs.len() - 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex::MemoryId;

    fn sample(title: &str) -> MemorySummary {
        MemorySummary {
            id: MemoryId::new(),
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

    #[test]
    fn domain_event_assimilation_requests_refresh() {
        let mut state = AppState::default();
        let action = state.apply_domain_event(DomainEvent::memory_assimilated(MemoryId::new(), 2));
        assert_eq!(action, TuiAction::RefreshList);
    }

    #[test]
    fn graph_summary_stores_hubs() {
        let mut state = AppState::default();
        let id = MemoryId::new();
        let action = state.apply_response(Response::GraphSummary {
            node_count: 3,
            edge_count: 2,
            hubs: vec![HubSummary {
                memory_id: id,
                title: "Hub".into(),
                inbound_links: 4,
            }],
        });
        assert_eq!(action, TuiAction::None);
        assert_eq!(state.graph_node_count, 3);
        assert_eq!(state.graph_hubs.len(), 1);
    }

    #[test]
    fn audit_log_stores_entries_and_chain_status() {
        let mut state = AppState::default();
        let action = state.apply_response(Response::AuditLog {
            entries: vec![AuditEvent {
                timestamp: "2026-01-01T00:00:00Z".into(),
                event_type: "assimilate".into(),
                details: "ok".into(),
                previous_hash: "GENESIS".into(),
                hash: "abc".into(),
            }],
            chain_intact: false,
        });
        assert_eq!(action, TuiAction::None);
        assert_eq!(state.audit_entries.len(), 1);
        assert!(!state.audit_chain_intact);
    }
}