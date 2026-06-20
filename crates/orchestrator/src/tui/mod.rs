//! Module TUI ratatui — compilé uniquement avec la feature `tui`.
//!
//! Consomme le même [`OrchestratorHandle`] et les enums [`Command`] / [`Response`] que le HUD egui.
//! Aucune logique métier : la présentation terminal est une « peau » interchangeable.

mod app;
mod state;
mod ui;

pub use app::TuiApp;
pub use state::{AppState, MemoryDetailView, View};
