//! Application egui — polling bridge, zéro logique métier.

use std::time::Instant;

use egui::{Context, TopBottomPanel};
use orchestrator::{
    BridgeError, ChannelHandle, Command, DomainEvent, OrchestratorHandle, OrchestratorThread,
};

use crate::list::show_virtual_memory_list;
use crate::metrics::FrameMetrics;
use crate::prefs::UiPreferences;
use crate::search_list::show_virtual_search_list;
use crate::state::{HudAction, HudMainView, HudState, LeftPanelMode, ToastKind};
use crate::theme::apply_theme;
use crate::views::{show_audit_view, show_chat_view, show_degraded_banner, show_graph_view};

/// Application HUD branchée sur le bridge orchestrateur.
pub struct HudApp {
    handle: ChannelHandle,
    thread: Option<OrchestratorThread>,
    event_rx: flume::Receiver<DomainEvent>,
    state: HudState,
    metrics: FrameMetrics,
    search_id: egui::Id,
    assimilate_id: egui::Id,
    last_frame: Option<Instant>,
    startup_sent: bool,
}

impl HudApp {
    /// Construit l'application avec handle bridge et thread orchestrateur.
    #[must_use]
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        handle: ChannelHandle,
        thread: OrchestratorThread,
    ) -> Self {
        let event_rx = handle.subscribe_events();
        let mut state = HudState::default();
        if let Some(storage) = cc.storage {
            if let Some(prefs) = UiPreferences::load_from_storage(storage) {
                prefs.apply_to(&mut state);
            }
        }
        Self {
            handle,
            thread: Some(thread),
            event_rx,
            state,
            metrics: FrameMetrics::default(),
            search_id: egui::Id::new("hud_search"),
            assimilate_id: egui::Id::new("hud_assimilate"),
            last_frame: None,
            startup_sent: false,
        }
    }

    fn send_command(&mut self, cmd: Command, label: &str) {
        self.state.set_busy(label);
        if let Err(BridgeError::ChannelClosed) = self.handle.send_command(cmd) {
            self.state
                .push_error("Bridge fermé — redémarrez l'application");
            self.state.busy = false;
            self.state.busy_label = None;
        }
    }

    fn poll_responses(&mut self) {
        loop {
            match self.handle.try_recv_response() {
                Ok(Some(response)) => {
                    let action = self.state.apply_response(response);
                    self.handle_action(action);
                }
                Ok(None) => break,
                Err(err) => {
                    if matches!(err, BridgeError::ChannelClosed) {
                        self.state.push_error("Canal réponses fermé");
                    }
                    break;
                }
            }
        }
    }

    fn poll_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            let action = self.state.apply_domain_event(event);
            self.handle_action(action);
        }
    }

    fn handle_action(&mut self, action: HudAction) {
        if action == HudAction::RefreshList {
            self.request_list();
        }
    }

    fn request_list(&mut self) {
        let filter = if self.state.list_filter.trim().is_empty() {
            None
        } else {
            Some(self.state.list_filter.trim().to_string())
        };
        self.send_command(
            Command::List {
                filter,
                offset: 0,
                limit: 10_000,
            },
            "Chargement liste…",
        );
    }

    fn request_search(&mut self) {
        if !self.state.embedding_available {
            self.state
                .push_error("Recherche indisponible — provider embeddings hors ligne");
            return;
        }
        let query = self.state.search_query.trim().to_string();
        if query.is_empty() {
            self.state.left_panel = LeftPanelMode::Memories;
            self.request_list();
            return;
        }
        self.send_command(Command::Search { query, limit: 50 }, "Recherche…");
    }

    fn request_assimilate(&mut self) {
        if !self.state.llm_available {
            self.state
                .push_error("LLM indisponible — assimilation désactivée");
            return;
        }
        let text = self.state.assimilate_text.trim().to_string();
        if text.is_empty() {
            self.state.push_error("Texte d'assimilation vide");
            return;
        }
        let tags = self.state.parsed_assimilate_tags();
        self.send_command(Command::Assimilate { text, tags }, "Assimilation LLM…");
    }

    fn request_detail(&mut self, id: String) {
        self.send_command(Command::GetMemory { id }, "Chargement détail…");
    }

    fn switch_main_view(&mut self, view: HudMainView) {
        if self.state.main_view == view {
            return;
        }
        self.state.main_view = view;
        match view {
            HudMainView::Explorer => self.request_list(),
            HudMainView::Graph => self.send_command(Command::Graph, "Chargement graphe…"),
            HudMainView::Audit => {
                self.send_command(Command::Audit { limit: 100 }, "Chargement audit…");
            }
            HudMainView::Chat => {}
        }
    }

    fn send_chat(&mut self) {
        if !self.state.llm_available {
            self.state
                .push_error("Chat indisponible — provider LLM hors ligne");
            return;
        }
        let message = self.state.chat_input.trim().to_string();
        if message.is_empty() {
            return;
        }
        self.send_command(Command::Chat { message }, "Chat…");
        self.state.chat_input.clear();
    }

    fn send_startup_commands(&mut self) {
        if self.startup_sent {
            return;
        }
        self.startup_sent = true;
        self.send_command(Command::HealthCheck, "Santé…");
        match self.state.main_view {
            HudMainView::Explorer => {
                if self.state.left_panel == LeftPanelMode::SearchResults
                    && !self.state.search_query.trim().is_empty()
                {
                    self.request_search();
                } else {
                    self.request_list();
                }
            }
            HudMainView::Graph => self.send_command(Command::Graph, "Chargement graphe…"),
            HudMainView::Audit => {
                self.send_command(Command::Audit { limit: 100 }, "Chargement audit…");
            }
            HudMainView::Chat => {}
        }
    }

    fn draw_view_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.state.main_view == HudMainView::Explorer, "Explorateur")
                .clicked()
            {
                self.switch_main_view(HudMainView::Explorer);
            }
            if ui
                .selectable_label(self.state.main_view == HudMainView::Graph, "Graphe")
                .clicked()
            {
                self.switch_main_view(HudMainView::Graph);
            }
            if ui
                .selectable_label(self.state.main_view == HudMainView::Audit, "Audit")
                .clicked()
            {
                self.switch_main_view(HudMainView::Audit);
            }
            if ui
                .selectable_label(self.state.main_view == HudMainView::Chat, "Chat")
                .clicked()
            {
                self.switch_main_view(HudMainView::Chat);
            }
        });
    }

    fn draw_top_bar(&mut self, ctx: &Context) {
        TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Orchestrateur");
                if let Some(ref version) = self.state.version {
                    ui.label(format!("v{version}"));
                }
                ui.separator();
                if self.state.busy {
                    ui.add(egui::Spinner::new());
                    if let Some(ref label) = self.state.busy_label {
                        ui.label(label);
                    }
                } else {
                    ui.label(&self.state.status);
                }
                if self.state.show_frame_metrics {
                    if let (Some(last), Some(avg), Some(p99)) = (
                        self.metrics.last(),
                        self.metrics.average(),
                        self.metrics.p99(),
                    ) {
                        ui.separator();
                        let color = if last > 10.0 {
                            egui::Color32::from_rgb(220, 80, 80)
                        } else {
                            ui.visuals().weak_text_color()
                        };
                        ui.colored_label(
                            color,
                            format!("{last:.1} ms (avg {avg:.1}, p99 {p99:.1})"),
                        );
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(if self.state.dark_mode { "☀" } else { "☾" })
                        .clicked()
                    {
                        self.state.dark_mode = !self.state.dark_mode;
                    }
                    if ui
                        .selectable_label(self.state.show_assimilate, "Assimiler")
                        .clicked()
                    {
                        self.state.show_assimilate = !self.state.show_assimilate;
                    }
                    if ui.button("Actualiser").clicked() {
                        self.request_list();
                    }
                    let busy = self.state.busy;
                    let search_ok = self.state.embedding_available;
                    if ui
                        .add_enabled(!busy && search_ok, egui::Button::new("Rechercher"))
                        .on_disabled_hover_text(
                            "Recherche indisponible — provider embeddings hors ligne",
                        )
                        .clicked()
                    {
                        self.request_search();
                    }
                });
            });

            self.draw_view_tabs(ui);

            if self.state.main_view == HudMainView::Explorer {
                ui.horizontal(|ui| {
                    ui.label("Filtre:");
                    let filter_response = ui.text_edit_singleline(&mut self.state.list_filter);
                    if filter_response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        self.request_list();
                    }
                    if ui.button("Appliquer filtre").clicked() {
                        self.request_list();
                    }
                    if self.state.left_panel == LeftPanelMode::SearchResults
                        && ui.button("← Liste complète").clicked()
                    {
                        self.state.left_panel = LeftPanelMode::Memories;
                        self.request_list();
                    }
                });
            }
        });
    }

    fn draw_assimilate_panel(&mut self, ctx: &Context) {
        if !self.state.show_assimilate {
            return;
        }
        TopBottomPanel::bottom("assimilate_panel")
            .resizable(true)
            .default_height(140.0)
            .show(ctx, |ui| {
                ui.heading("Assimilation");
                ui.horizontal(|ui| {
                    ui.label("Tags:");
                    ui.text_edit_singleline(&mut self.state.assimilate_tags)
                        .on_hover_text("Séparés par des virgules, ex: rust, architecture");
                });
                ui.add(
                    egui::TextEdit::multiline(&mut self.state.assimilate_text)
                        .id(self.assimilate_id)
                        .desired_rows(4)
                        .hint_text("Texte brut à assimiler via le provider LLM…"),
                );
                ui.horizontal(|ui| {
                    let busy = self.state.busy;
                    let llm_ok = self.state.llm_available;
                    if ui
                        .add_enabled(!busy && llm_ok, egui::Button::new("Lancer assimilation"))
                        .on_disabled_hover_text("LLM indisponible — assimilation désactivée")
                        .clicked()
                    {
                        self.request_assimilate();
                    }
                    if ui.button("Effacer").clicked() {
                        self.state.assimilate_text.clear();
                        self.state.assimilate_tags.clear();
                    }
                });
            });
    }

    fn draw_search_panel(&mut self, ctx: &Context) {
        let search_ok = self.state.embedding_available;
        let accent = if search_ok {
            ui_visual_weak(ctx)
        } else {
            egui::Color32::from_rgb(220, 80, 80)
        };

        TopBottomPanel::bottom("search_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let label = if search_ok {
                    "Recherche:"
                } else {
                    "Recherche (indisponible):"
                };
                ui.colored_label(accent, label);
                let hint = if search_ok {
                    "Ctrl+K — recherche sémantique"
                } else {
                    "Provider embeddings hors ligne — liste et détail restent disponibles"
                };
                let text_edit = egui::TextEdit::singleline(&mut self.state.search_query)
                    .id(self.search_id)
                    .hint_text(hint);
                let response = if search_ok {
                    ui.add(text_edit)
                } else {
                    ui.add_enabled(false, text_edit)
                };
                if self.state.focus_search && search_ok {
                    response.request_focus();
                    self.state.focus_search = false;
                }
                if search_ok
                    && (ui.button("Go").clicked()
                        || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                {
                    self.request_search();
                }
            });
        });
    }

    fn draw_toasts(&self, ctx: &Context) {
        egui::Area::new(egui::Id::new("toasts"))
            .fixed_pos(egui::pos2(12.0, 56.0))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    for toast in &self.state.toasts {
                        let fill = match toast.kind {
                            ToastKind::Error => ui.visuals().error_fg_color.gamma_multiply(0.18),
                            ToastKind::Success => ui.visuals().warn_fg_color.gamma_multiply(0.12),
                            ToastKind::Info => ui.visuals().weak_text_color().gamma_multiply(0.15),
                        };
                        egui::Frame::popup(ui.style()).fill(fill).show(ui, |ui| {
                            ui.label(&toast.message);
                        });
                    }
                });
            });
    }

    fn draw_left_panel(&mut self, ui: &mut egui::Ui) {
        match self.state.left_panel {
            LeftPanelMode::Memories => {
                ui.heading(format!("Mémoires ({})", self.state.total));
                let mut clicked_id = None;
                show_virtual_memory_list(
                    ui,
                    &self.state.memories,
                    self.state.selected_id.as_deref(),
                    &mut |memory| {
                        clicked_id = Some(memory.id.to_string());
                    },
                );
                if let Some(id) = clicked_id {
                    self.state.selected_id = Some(id.clone());
                    self.request_detail(id);
                }
            }
            LeftPanelMode::SearchResults => {
                ui.heading(format!("Résultats ({})", self.state.search_hits.len()));
                let mut clicked_id = None;
                show_virtual_search_list(
                    ui,
                    &self.state.search_hits,
                    self.state.selected_id.as_deref(),
                    &mut |hit| {
                        clicked_id = Some(hit.id.clone());
                    },
                );
                if let Some(id) = clicked_id {
                    self.state.selected_id = Some(id.clone());
                    self.request_detail(id);
                }
            }
        }
    }

    fn draw_central(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| match self.state.main_view {
            HudMainView::Explorer => {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(ui.available_width() * 0.45);
                        self.draw_left_panel(ui);
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.heading("Détail");
                        if let Some(ref detail) = self.state.detail {
                            ui.label(format!("Titre: {}", detail.title));
                            if !detail.tags.is_empty() {
                                ui.label(format!("Tags: {}", detail.tags.join(", ")));
                            }
                            ui.separator();
                            egui::ScrollArea::vertical()
                                .auto_shrink([false; 2])
                                .show(ui, |ui| {
                                    ui.label(&detail.content);
                                });
                        } else {
                            ui.label("Sélectionnez une mémoire ou lancez une recherche.");
                        }
                    });
                });
            }
            HudMainView::Graph => {
                let mut selected = None;
                show_graph_view(
                    ui,
                    self.state.graph_node_count,
                    self.state.graph_edge_count,
                    &self.state.graph_hubs,
                    &mut |id| selected = Some(id.to_string()),
                );
                if let Some(id) = selected {
                    self.state.main_view = HudMainView::Explorer;
                    self.state.selected_id = Some(id.clone());
                    self.request_detail(id);
                }
            }
            HudMainView::Audit => {
                show_audit_view(
                    ui,
                    &self.state.audit_entries,
                    self.state.audit_chain_intact,
                );
            }
            HudMainView::Chat => {
                let mut send = false;
                show_chat_view(
                    ui,
                    &mut self.state.chat_input,
                    self.state.chat_reply.as_deref(),
                    self.state.llm_available,
                    &mut || send = true,
                );
                if send {
                    self.send_chat();
                }
            }
        });
    }

    fn handle_shortcuts(&mut self, ctx: &Context) {
        ctx.input(|i| {
            if i.modifiers.ctrl && i.key_pressed(egui::Key::K) && self.state.embedding_available {
                self.state.focus_search = true;
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::N) {
                self.state.show_assimilate = true;
            }
            if i.key_pressed(egui::Key::Escape) {
                self.state.detail = None;
                self.state.selected_id = None;
            }
        });
    }

    fn record_frame_time(&mut self, ctx: &Context) {
        let now = Instant::now();
        if let Some(prev) = self.last_frame.replace(now) {
            let ms = now.duration_since(prev).as_secs_f32() * 1000.0;
            self.metrics.record(ms);
            if ms > 12.0 {
                tracing::debug!(
                    frame_ms = ms,
                    memories = self.state.memories.len(),
                    hits = self.state.search_hits.len(),
                    "hud frame slow"
                );
            }
        }
        ctx.request_repaint();
    }
}

impl eframe::App for HudApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        UiPreferences::from_state(&self.state).save_to_storage(storage);
    }

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx, self.state.dark_mode);
        self.handle_shortcuts(ctx);
        show_degraded_banner(
            ctx,
            self.state.embedding_available,
            self.state.llm_available,
        );
        self.send_startup_commands();
        self.poll_responses();
        self.poll_events();
        self.state.tick_toasts();

        self.draw_top_bar(ctx);
        self.draw_assimilate_panel(ctx);
        self.draw_search_panel(ctx);
        self.draw_central(ctx);
        self.draw_toasts(ctx);
        self.record_frame_time(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(thread) = self.thread.take() {
            drop(self.handle.clone());
            thread.join();
        }
    }
}

fn ui_visual_weak(ctx: &Context) -> egui::Color32 {
    ctx.style().visuals.weak_text_color()
}

impl Drop for HudApp {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            drop(self.handle.clone());
            thread.join();
        }
    }
}
