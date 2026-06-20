//! État UI pur — testable sans egui, sans logique métier.

use orchestrator::{
    format_health_status, AuditEvent, BridgeSearchHit, DomainEvent, HubSummary, MemoryDetailView,
    MemorySummary, Response,
};

/// Hit de recherche affiché dans le panneau gauche.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchHitView {
    /// Identifiant mémoire.
    pub id: String,
    /// Score de similarité.
    pub score: f32,
    /// Extrait optionnel.
    pub snippet: Option<String>,
}

impl SearchHitView {
    /// Construit une vue depuis un hit bridge.
    #[must_use]
    pub fn from_hit(hit: &BridgeSearchHit) -> Self {
        Self {
            id: hit.memory_id.to_string(),
            score: hit.score,
            snippet: hit.snippet.clone(),
        }
    }
}

/// Vue principale du HUD (onglets).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HudMainView {
    /// Explorateur mémoires + détail.
    #[default]
    Explorer,
    /// Graphe de connaissances.
    Graph,
    /// Journal d'audit.
    Audit,
    /// Chat libre LLM.
    Chat,
}

/// Panneau gauche actif.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LeftPanelMode {
    /// Liste complète des mémoires.
    #[default]
    Memories,
    /// Résultats de recherche vectorielle.
    SearchResults,
}

/// Action suggérée à la couche egui après application d'une réponse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HudAction {
    /// Aucune action supplémentaire.
    #[default]
    None,
    /// Recharger la liste des mémoires.
    RefreshList,
}

/// Catégorie visuelle d'un toast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    /// Information neutre.
    Info,
    /// Succès.
    Success,
    /// Erreur.
    Error,
}

/// Toast non-bloquant affiché en overlay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    /// Message utilisateur.
    pub message: String,
    /// Frames restantes avant disparition.
    pub ttl_frames: u32,
    /// Style du toast.
    pub kind: ToastKind,
}

/// État mutable du HUD (polling bridge uniquement).
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct HudState {
    /// Résumés pour la liste virtualisée.
    pub memories: Vec<MemorySummary>,
    /// Total après filtrage côté orchestrateur.
    pub total: usize,
    /// Hits de recherche courants.
    pub search_hits: Vec<SearchHitView>,
    /// Mode du panneau gauche.
    pub left_panel: LeftPanelMode,
    /// Identifiant sélectionné (string UUID).
    pub selected_id: Option<String>,
    /// Détail affiché dans le panneau droit.
    pub detail: Option<MemoryDetailView>,
    /// Requête de recherche en cours de saisie.
    pub search_query: String,
    /// Filtre liste en cours de saisie.
    pub list_filter: String,
    /// Texte à assimiler (panneau dédié).
    pub assimilate_text: String,
    /// Tags suggérés, séparés par virgules.
    pub assimilate_tags: String,
    /// Panneau assimilation déplié.
    pub show_assimilate: bool,
    /// Message barre de statut.
    pub status: String,
    /// Version orchestrateur (health check).
    pub version: Option<String>,
    /// Requête async en vol.
    pub busy: bool,
    /// Libellé de l'opération en cours.
    pub busy_label: Option<String>,
    /// Notifications éphémères.
    pub toasts: Vec<Toast>,
    /// Mode sombre actif.
    pub dark_mode: bool,
    /// Focus recherche demandé (raccourci clavier).
    pub focus_search: bool,
    /// Afficher les métriques frame dans la barre.
    pub show_frame_metrics: bool,
    /// Provider LLM joignable (assimilation / chat).
    pub llm_available: bool,
    /// Provider embeddings joignable (recherche sémantique).
    pub embedding_available: bool,
    /// Onglet principal actif.
    pub main_view: HudMainView,
    /// Nombre de nœuds du graphe (vue Graph).
    pub graph_node_count: usize,
    /// Nombre d'arêtes du graphe (vue Graph).
    pub graph_edge_count: usize,
    /// Hubs du graphe (vue Graph).
    pub graph_hubs: Vec<HubSummary>,
    /// Entrées d'audit (vue Audit).
    pub audit_entries: Vec<AuditEvent>,
    /// Chaîne d'audit intacte.
    pub audit_chain_intact: bool,
    /// Saisie chat en cours.
    pub chat_input: String,
    /// Dernière réponse LLM affichée.
    pub chat_reply: Option<String>,
}

impl Default for HudState {
    fn default() -> Self {
        Self {
            memories: Vec::new(),
            total: 0,
            search_hits: Vec::new(),
            left_panel: LeftPanelMode::Memories,
            selected_id: None,
            detail: None,
            search_query: String::new(),
            list_filter: String::new(),
            assimilate_text: String::new(),
            assimilate_tags: String::new(),
            show_assimilate: false,
            status: "Prêt".to_string(),
            version: None,
            busy: false,
            busy_label: None,
            toasts: Vec::new(),
            dark_mode: true,
            focus_search: false,
            show_frame_metrics: true,
            llm_available: true,
            embedding_available: true,
            main_view: HudMainView::Explorer,
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

impl HudState {
    /// Applique une [`Response`] bridge sans logique domaine.
    #[must_use]
    pub fn apply_response(&mut self, response: Response) -> HudAction {
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
                self.status =
                    format_health_status(&status, llm_available, embedding_available);
                self.clear_busy();
                HudAction::None
            }
            Response::MemoryList { items, total } => {
                self.memories = items;
                self.total = total;
                self.left_panel = LeftPanelMode::Memories;
                self.status = format!("{total} mémoire(s)");
                self.clear_busy();
                HudAction::None
            }
            Response::MemoryDetail { memory } => {
                self.selected_id = Some(memory.id.to_string());
                self.detail = Some(MemoryDetailView::from_memory(&memory));
                self.status = "Détail chargé".to_string();
                self.clear_busy();
                HudAction::None
            }
            Response::SearchResults { items } => {
                self.search_hits = items.iter().map(SearchHitView::from_hit).collect();
                self.left_panel = LeftPanelMode::SearchResults;
                let count = self.search_hits.len();
                self.status = format!("{count} résultat(s) de recherche");
                self.clear_busy();
                self.push_info(format!("Recherche : {count} hit(s)"));
                HudAction::None
            }
            Response::Event(event) => self.apply_domain_event(event),
            Response::Error(err) => {
                self.status.clone_from(&err.message);
                self.clear_busy();
                self.push_error(format!(
                    "[{kind}] {msg}",
                    kind = err.kind,
                    msg = err.message
                ));
                HudAction::None
            }
            Response::Success { message } => {
                self.status = message;
                self.clear_busy();
                HudAction::None
            }
            Response::Assimilated { memory_id, title } => {
                self.selected_id = Some(memory_id.to_string());
                self.status = format!("Assimilé : {title}");
                self.clear_busy();
                self.push_success(format!("Assimilation réussie — {title}"));
                HudAction::RefreshList
            }
            Response::GraphSummary {
                node_count,
                edge_count,
                hubs,
            } => self.apply_graph_summary(node_count, edge_count, hubs),
            Response::AuditLog {
                entries,
                chain_intact,
            } => self.apply_audit_log(entries, chain_intact),
            Response::ChatReply { reply } => self.apply_chat_reply(reply),
            Response::SkillList { skills } => self.apply_skill_list(&skills),
            Response::SkillResult { message } => self.apply_skill_result(message),
        }
    }

    fn apply_graph_summary(
        &mut self,
        node_count: usize,
        edge_count: usize,
        hubs: Vec<orchestrator::HubSummary>,
    ) -> HudAction {
        self.graph_node_count = node_count;
        self.graph_edge_count = edge_count;
        self.graph_hubs = hubs;
        self.status = format!("Graphe : {node_count} nœuds, {edge_count} arêtes");
        self.clear_busy();
        HudAction::None
    }

    fn apply_audit_log(
        &mut self,
        entries: Vec<orchestrator::AuditEvent>,
        chain_intact: bool,
    ) -> HudAction {
        self.audit_entries = entries;
        self.audit_chain_intact = chain_intact;
        let chain = if chain_intact { "intacte" } else { "ROMPUE" };
        self.status = format!(
            "Audit : {} entrée(s), chaîne {chain}",
            self.audit_entries.len()
        );
        self.clear_busy();
        if !chain_intact {
            self.push_error("Chaîne d'audit compromise — vérification requise");
        }
        HudAction::None
    }

    fn apply_chat_reply(&mut self, reply: String) -> HudAction {
        self.chat_reply = Some(reply);
        self.status = "Réponse chat reçue".to_string();
        self.clear_busy();
        HudAction::None
    }

    fn apply_skill_list(&mut self, skills: &[orchestrator::SkillSummary]) -> HudAction {
        self.status = format!("{} skill(s) disponibles", skills.len());
        self.clear_busy();
        self.push_info(format!("Skills : {}", skills.len()));
        HudAction::None
    }

    fn apply_skill_result(&mut self, message: String) -> HudAction {
        self.status = "Skill exécutée".to_string();
        self.clear_busy();
        self.push_success(message);
        HudAction::None
    }

    /// Applique un événement domaine poussé par le fan-out.
    #[must_use]
    pub fn apply_domain_event(&mut self, event: DomainEvent) -> HudAction {
        match event {
            DomainEvent::MemoryAssimilated(payload) => {
                self.push_success(format!("Assimilation réussie ({})", payload.memory_id));
                HudAction::RefreshList
            }
            DomainEvent::KnowledgeGraphValidated(payload) => {
                self.push_info(format!(
                    "Graphe validé — {} nœuds, {} arêtes",
                    payload.node_count, payload.edge_count
                ));
                HudAction::None
            }
        }
    }

    /// Marque une opération async en cours.
    pub fn set_busy(&mut self, label: impl Into<String>) {
        self.busy = true;
        self.busy_label = Some(label.into());
    }

    fn clear_busy(&mut self) {
        self.busy = false;
        self.busy_label = None;
    }

    /// Parse les tags d'assimilation (virgules, trim).
    #[must_use]
    pub fn parsed_assimilate_tags(&self) -> Vec<String> {
        self.assimilate_tags
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
            .collect()
    }

    /// Ajoute un toast informatif.
    pub fn push_info(&mut self, message: impl Into<String>) {
        self.push_toast(message, ToastKind::Info);
    }

    /// Ajoute un toast de succès.
    pub fn push_success(&mut self, message: impl Into<String>) {
        self.push_toast(message, ToastKind::Success);
    }

    /// Ajoute un toast d'erreur.
    pub fn push_error(&mut self, message: impl Into<String>) {
        self.push_toast(message, ToastKind::Error);
    }

    /// Ajoute un toast avec durée par défaut (~3 s à 60 fps).
    pub fn push_toast(&mut self, message: impl Into<String>, kind: ToastKind) {
        self.toasts.push(Toast {
            message: message.into(),
            ttl_frames: 180,
            kind,
        });
    }

    /// Décrémente les TTL et retire les toasts expirés.
    pub fn tick_toasts(&mut self) {
        self.toasts.retain_mut(|toast| {
            toast.ttl_frames = toast.ttl_frames.saturating_sub(1);
            toast.ttl_frames > 0
        });
    }
}

#[cfg(test)]
mod tests {
    use cortex::MemoryId;

    use super::*;
    use orchestrator::Response;

    #[test]
    fn apply_health_sets_version() {
        let mut state = HudState::default();
        let _ = state.apply_response(Response::Health {
            status: "ok".into(),
            version: "0.4.0".into(),
            llm_available: true,
            embedding_available: true,
        });
        assert_eq!(state.version.as_deref(), Some("0.4.0"));
        assert!(state.llm_available);
        assert!(!state.busy);
    }

    #[test]
    fn apply_memory_list_updates_items() {
        let mut state = HudState::default();
        let _ = state.apply_response(Response::MemoryList {
            items: vec![],
            total: 0,
        });
        assert_eq!(state.total, 0);
        assert!(state.memories.is_empty());
        assert_eq!(state.left_panel, LeftPanelMode::Memories);
    }

    #[test]
    fn apply_search_results_switches_panel() {
        let mut state = HudState::default();
        let _ = state.apply_response(Response::SearchResults {
            items: vec![orchestrator::BridgeSearchHit {
                memory_id: MemoryId::new(),
                score: 0.92,
                snippet: Some("extrait".into()),
            }],
        });
        assert_eq!(state.left_panel, LeftPanelMode::SearchResults);
        assert_eq!(state.search_hits.len(), 1);
    }

    #[test]
    fn apply_error_pushes_toast() {
        let mut state = HudState::default();
        let _ = state.apply_response(Response::Error(orchestrator::AppError {
            kind: "validation".into(),
            message: "rejeté".into(),
        }));
        assert_eq!(state.toasts.len(), 1);
        assert_eq!(state.toasts[0].kind, ToastKind::Error);
        assert!(state.toasts[0].message.contains("rejeté"));
    }

    #[test]
    fn domain_event_assimilation_requests_refresh() {
        let mut state = HudState::default();
        let action = state.apply_domain_event(DomainEvent::memory_assimilated(MemoryId::new(), 2));
        assert_eq!(action, HudAction::RefreshList);
    }

    #[test]
    fn parsed_assimilate_tags_splits_commas() {
        let mut state = HudState::default();
        state.assimilate_tags = " rust , egui, ".into();
        let tags = state.parsed_assimilate_tags();
        assert_eq!(tags, vec!["rust", "egui"]);
    }
}
