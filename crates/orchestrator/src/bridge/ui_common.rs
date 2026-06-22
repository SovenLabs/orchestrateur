//! Helpers d'état client bridge — sans dépendance graphique.

use cortex::Memory;

/// DTO détail mémoire pour les vues HUD et TUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryDetailView {
    /// Identifiant UUID string.
    pub id: String,
    /// Titre affiché.
    pub title: String,
    /// Corps markdown.
    pub content: String,
    /// Tags normalisés.
    pub tags: Vec<String>,
}

impl MemoryDetailView {
    /// Construit une vue depuis une entité [`Memory`] domaine.
    #[must_use]
    pub fn from_memory(memory: &Memory) -> Self {
        Self {
            id: memory.id.to_string(),
            title: memory.title.clone(),
            content: memory.content.clone(),
            tags: memory
                .tags
                .iter()
                .map(|tag| tag.as_str().to_string())
                .collect(),
        }
    }
}

/// Formate le libellé de santé (ok ou dégradé) pour barres de statut HUD/TUI.
#[must_use]
pub fn format_health_status(
    status: &str,
    llm_available: bool,
    embedding_available: bool,
) -> String {
    if status == "ok" {
        format!("Santé: {status}")
    } else {
        let mut parts = vec!["Santé: dégradée".to_string()];
        if !llm_available {
            parts.push("LLM indisponible".into());
        }
        if !embedding_available {
            parts.push("recherche indisponible".into());
        }
        parts.join(" — ")
    }
}

#[cfg(test)]
mod tests {
    use cortex::Memory;

    use super::*;

    #[test]
    fn from_memory_maps_fields() {
        let memory = Memory::new("Titre", "Corps").expect("memory");
        let view = MemoryDetailView::from_memory(&memory);
        assert_eq!(view.title, "Titre");
        assert_eq!(view.content, "Corps");
        assert_eq!(view.id, memory.id.to_string());
    }

    #[test]
    fn health_ok_short() {
        assert_eq!(
            format_health_status("ok", true, true),
            "Santé: ok"
        );
    }

    #[test]
    fn health_degraded_lists_missing_services() {
        let msg = format_health_status("degraded", false, false);
        assert!(msg.contains("dégradée"));
        assert!(msg.contains("LLM"));
        assert!(msg.contains("recherche"));
    }
}