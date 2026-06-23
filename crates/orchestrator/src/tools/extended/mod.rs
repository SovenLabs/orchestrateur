//! Outils agent étendus — fichiers, skills, sessions, shell, orchestration.

mod external;
mod files;
mod interaction;
mod orchestration;
mod session_search;
mod shell;
mod skills;
mod state;

use std::sync::Arc;

use super::registry::ToolRegistry;

/// Enregistre les outils étendus dans le registre.
pub fn register_all(registry: &mut ToolRegistry) {
    files::register(registry);
    skills::register(registry);
    registry.register(Arc::new(session_search::SessionSearchTool));
    state::register(registry);
    shell::register(registry);
    registry.register(Arc::new(interaction::ClarifyTool));
    orchestration::register(registry);
    external::register(registry);
}

/// Sérialise une réponse JSON pour le modèle.
pub(crate) fn json_result(value: &impl serde::Serialize) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
}