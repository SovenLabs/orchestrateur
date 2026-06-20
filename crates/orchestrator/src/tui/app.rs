//! Boucle événements TUI — bridge uniquement, zéro logique métier.

use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use cortex::DomainEvent;
use flume::Receiver;

use crate::bridge::{BridgeError, Command, OrchestratorHandle};
use crate::{ChannelHandle, OrchestratorThread};

use super::state::{AppState, TuiAction, View};
use super::ui;

/// Erreur TUI (terminal ou bridge).
#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    /// Erreur I/O terminal.
    #[error("terminal: {0}")]
    Io(#[from] io::Error),
    /// Bridge fermé.
    #[error("bridge: {0}")]
    Bridge(#[from] BridgeError),
}

/// Application TUI branchée sur le bridge orchestrateur.
pub struct TuiApp {
    handle: ChannelHandle,
    thread: Option<OrchestratorThread>,
    event_rx: Receiver<DomainEvent>,
    state: AppState,
}

impl TuiApp {
    /// Construit le TUI avec handle et thread orchestrateur.
    #[must_use]
    pub fn new(handle: ChannelHandle, thread: OrchestratorThread) -> Self {
        let event_rx = handle.subscribe_events();
        Self {
            handle,
            thread: Some(thread),
            event_rx,
            state: AppState::default(),
        }
    }

    /// Lance le TUI (setup terminal, boucle, cleanup garanti).
    ///
    /// # Errors
    ///
    /// Propage [`TuiError`] si le terminal ou le rendu échoue.
    pub fn run(&mut self) -> Result<(), TuiError> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        self.send_startup_commands();
        let result = self.run_event_loop(&mut terminal);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        if let Some(thread) = self.thread.take() {
            drop(self.handle.clone());
            thread.join();
        }

        result
    }

    fn run_event_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), TuiError> {
        while !self.state.should_quit {
            self.poll_bridge();
            self.poll_events();
            terminal.draw(|frame| ui::draw(frame, &self.state))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('c')
                    {
                        self.state.should_quit = true;
                        continue;
                    }
                    self.handle_key_event(key);
                }
            }
        }
        Ok(())
    }

    fn send_startup_commands(&mut self) {
        let _ = self.send_command(Command::HealthCheck);
        self.request_list();
    }

    fn send_command(&self, cmd: Command) -> Result<(), BridgeError> {
        self.handle.send_command(cmd)
    }

    fn poll_bridge(&mut self) {
        loop {
            match self.handle.try_recv_response() {
                Ok(Some(response)) => {
                    let action = self.state.apply_response(response);
                    self.handle_action(action);
                }
                Err(BridgeError::ChannelClosed) => {
                    self.state.status_message = "Bridge fermé".to_string();
                    self.state.should_quit = true;
                    break;
                }
                Ok(None) | Err(_) => break,
            }
        }
    }

    fn poll_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            let action = self.state.apply_domain_event(event);
            self.handle_action(action);
        }
    }

    fn handle_action(&mut self, action: TuiAction) {
        if action == TuiAction::RefreshList {
            self.request_list();
        }
    }

    fn request_list(&mut self) {
        let filter = if self.state.input_buffer.is_empty() || self.state.search_mode {
            None
        } else {
            Some(self.state.input_buffer.clone())
        };
        if let Err(err) = self.send_command(Command::List {
            filter,
            offset: 0,
            limit: 10_000,
        }) {
            self.state.status_message = format!("Erreur list: {err}");
        } else {
            self.state.status_message = "Chargement…".to_string();
        }
    }

    fn request_search(&mut self) {
        let query = self.state.input_buffer.trim().to_string();
        if query.is_empty() {
            self.request_list();
            return;
        }
        if let Err(err) = self.send_command(Command::Search { query, limit: 50 }) {
            self.state.status_message = format!("Erreur search: {err}");
        } else {
            self.state.status_message = "Recherche…".to_string();
        }
    }

    fn open_selected_detail(&mut self) {
        let Some(summary) = self.state.selected().cloned() else {
            return;
        };
        let id = summary.id.to_string();
        if let Err(err) = self.send_command(Command::GetMemory { id }) {
            self.state.status_message = format!("Erreur get: {err}");
        } else {
            self.state.status_message = "Chargement détail…".to_string();
        }
    }

    fn submit_assimilate(&mut self) {
        let text = self.state.assimilate_text.trim().to_string();
        if text.is_empty() {
            return;
        }
        if let Err(err) = self.send_command(Command::Assimilate {
            text,
            tags: vec!["from-tui".into()],
        }) {
            self.state.status_message = format!("Erreur assimilate: {err}");
        } else {
            self.state.status_message = "Assimilation envoyée…".to_string();
            self.request_list();
        }
        self.state.assimilate_text.clear();
        self.state.current_view = View::List;
    }

    fn request_graph(&mut self) {
        if let Err(err) = self.send_command(Command::Graph) {
            self.state.status_message = format!("Erreur graph: {err}");
        } else {
            self.state.current_view = View::Graph;
            self.state.status_message = "Chargement graphe…".to_string();
        }
    }

    fn request_audit(&mut self) {
        if let Err(err) = self.send_command(Command::Audit { limit: 50 }) {
            self.state.status_message = format!("Erreur audit: {err}");
        } else {
            self.state.current_view = View::Audit;
            self.state.status_message = "Chargement audit…".to_string();
        }
    }

    fn open_selected_hub(&mut self) {
        let Some(hub) = self.state.selected_hub().cloned() else {
            return;
        };
        let id = hub.memory_id.to_string();
        if let Err(err) = self.send_command(Command::GetMemory { id }) {
            self.state.status_message = format!("Erreur get: {err}");
        } else {
            self.state.status_message = "Chargement détail…".to_string();
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match self.state.current_view {
            View::List => self.handle_list_keys(key),
            View::Detail => self.handle_detail_keys(key),
            View::Assimilate => self.handle_assimilate_keys(key),
            View::Graph => self.handle_graph_keys(key),
            View::Audit => self.handle_audit_keys(key),
            View::Chat => self.handle_chat_keys(key),
            View::Help => {
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('?')) {
                    self.state.current_view = View::List;
                }
            }
        }
    }

    fn handle_list_keys(&mut self, key: KeyEvent) {
        if self.state.search_mode {
            self.handle_search_input(key);
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.state.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => self.state.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.state.select_previous(),
            KeyCode::Enter => self.open_selected_detail(),
            KeyCode::Char('/') => {
                if !self.state.embedding_available {
                    self.state.status_message =
                        "Recherche indisponible — provider embeddings hors ligne".to_string();
                    return;
                }
                self.state.search_mode = true;
                self.state.input_buffer.clear();
                self.state.status_message =
                    "Recherche sémantique — Entrée valide, Esc annule".to_string();
            }
            KeyCode::Char('a') => {
                if !self.state.llm_available {
                    self.state.status_message =
                        "LLM indisponible — assimilation désactivée".to_string();
                    return;
                }
                self.state.current_view = View::Assimilate;
                self.state.assimilate_text.clear();
            }
            KeyCode::Char('r') => self.request_list(),
            KeyCode::Char('g') => self.request_graph(),
            KeyCode::Char('u') => self.request_audit(),
            KeyCode::Char('c') => {
                if !self.state.llm_available {
                    self.state.status_message =
                        "LLM indisponible — chat désactivé".to_string();
                    return;
                }
                self.state.current_view = View::Chat;
                self.state.chat_input.clear();
            }
            KeyCode::Char('?') => self.state.current_view = View::Help,
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.input_buffer.push(c);
                self.state.apply_local_filter();
            }
            KeyCode::Backspace => {
                self.state.input_buffer.pop();
                self.state.apply_local_filter();
            }
            KeyCode::Esc if !self.state.input_buffer.is_empty() => {
                self.state.input_buffer.clear();
                self.state.apply_local_filter();
            }
            _ => {}
        }
    }

    fn handle_search_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.state.search_mode = false;
                self.state.input_buffer.clear();
                self.request_list();
            }
            KeyCode::Enter => {
                self.state.search_mode = false;
                self.request_search();
            }
            KeyCode::Char(c) => self.state.input_buffer.push(c),
            KeyCode::Backspace => {
                self.state.input_buffer.pop();
            }
            _ => {}
        }
    }

    fn handle_detail_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.state.current_view = View::List,
            KeyCode::Char('a') => {
                self.state.current_view = View::Assimilate;
                self.state.assimilate_text.clear();
            }
            KeyCode::Char('?') => self.state.current_view = View::Help,
            _ => {}
        }
    }

    fn handle_assimilate_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.state.assimilate_text.clear();
                self.state.current_view = View::List;
            }
            KeyCode::Enter => self.submit_assimilate(),
            KeyCode::Char(c) => self.state.assimilate_text.push(c),
            KeyCode::Backspace => {
                self.state.assimilate_text.pop();
            }
            _ => {}
        }
    }

    fn handle_graph_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.state.current_view = View::List,
            KeyCode::Char('j') | KeyCode::Down => self.state.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.state.select_previous(),
            KeyCode::Enter => self.open_selected_hub(),
            KeyCode::Char('r') => self.request_graph(),
            KeyCode::Char('?') => self.state.current_view = View::Help,
            _ => {}
        }
    }

    fn handle_audit_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.state.current_view = View::List,
            KeyCode::Char('r') => self.request_audit(),
            KeyCode::Char('?') => self.state.current_view = View::Help,
            _ => {}
        }
    }

    fn submit_chat(&mut self) {
        let message = self.state.chat_input.trim().to_string();
        if message.is_empty() {
            return;
        }
        if let Err(err) = self.send_command(Command::Chat { message }) {
            self.state.status_message = format!("Erreur chat: {err}");
        } else {
            self.state.status_message = "Chat envoyé…".to_string();
        }
        self.state.chat_input.clear();
    }

    fn handle_chat_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.state.current_view = View::List,
            KeyCode::Enter => self.submit_chat(),
            KeyCode::Char('?') => self.state.current_view = View::Help,
            KeyCode::Char(c) => self.state.chat_input.push(c),
            KeyCode::Backspace => {
                self.state.chat_input.pop();
            }
            _ => {}
        }
    }
}
