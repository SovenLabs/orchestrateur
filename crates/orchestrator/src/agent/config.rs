use crate::config::AgentSettingsConfig;

/// Configuration de la boucle agent Phase 7–10.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentConfig {
    /// Nombre maximal d'itérations outil par tour.
    pub max_tool_iterations: usize,
    /// Injecte le contexte graphe (hubs) dans le prompt système.
    pub graph_context_enabled: bool,
    /// Nombre de hubs affichés dans le contexte graphe.
    pub graph_hub_limit: usize,
    /// Recherche mémoire proactive avant le premier appel LLM.
    pub proactive_memory_search: bool,
    /// Limite de résultats pour la recherche proactive.
    pub proactive_search_limit: usize,
    /// Assimile automatiquement un résumé du tour en fin de boucle (Phase 10 : défaut `true`).
    pub auto_assimilate_turn: bool,
    /// Nombre maximal de tours d'historique envoyés au LLM.
    pub max_history_turns: usize,
    /// Toolset actif (`memory`, `agent`, `full`, …) — Phase 10.
    pub active_toolset: String,
    /// Expose `skill_list` / `skill_execute` à l'agent (Phase 12).
    pub skill_tools_enabled: bool,
    /// Injecte le catalogue skills dans le prompt (Phase 13).
    pub skill_auto_suggest: bool,
    /// Exécute automatiquement la skill la mieux notée (Phase 14).
    pub skill_auto_execute: bool,
    /// Score minimal pour l'auto-exécution (Phase 14).
    pub skill_auto_execute_threshold: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self::from_settings(&AgentSettingsConfig::default())
    }
}

impl AgentConfig {
    /// Projette la configuration TOML applicative vers la config runtime agent.
    #[must_use]
    pub fn from_settings(settings: &AgentSettingsConfig) -> Self {
        Self {
            max_tool_iterations: settings.max_tool_iterations,
            graph_context_enabled: settings.graph_context_enabled,
            graph_hub_limit: settings.graph_hub_limit,
            proactive_memory_search: settings.proactive_memory_search,
            proactive_search_limit: settings.proactive_search_limit,
            auto_assimilate_turn: settings.auto_assimilate_turn,
            max_history_turns: settings.max_history_turns,
            active_toolset: settings.active_toolset.clone(),
            skill_tools_enabled: settings.skill_tools_enabled,
            skill_auto_suggest: settings.skill_auto_suggest,
            skill_auto_execute: settings.skill_auto_execute,
            skill_auto_execute_threshold: settings.skill_auto_execute_threshold,
        }
    }
}