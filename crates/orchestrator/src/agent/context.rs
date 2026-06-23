use cortex::AgentContext;
use crate::skills::suggest_skills;
use crate::skills::SkillRegistry;
use crate::tools::ToolRegistry;

use super::config::AgentConfig;

/// Formate un [`AgentContext`] en sections texte pour le prompt système.
#[must_use]
pub fn format_agent_context(ctx: &AgentContext) -> String {
    let mut sections = Vec::new();
    if let Some(graph) = &ctx.graph_context {
        sections.push(format!("## Graphe de connaissances\n{graph}"));
    }
    if !ctx.memories.is_empty() {
        let lines: Vec<String> = ctx
            .memories
            .iter()
            .map(|m| format!("- [{}] {}: {}", m.id, m.title, truncate_memory(&m.content, 200)))
            .collect();
        sections.push(format!(
            "## Souvenirs pertinents (recherche proactive)\n{}",
            lines.join("\n")
        ));
    }
    if sections.is_empty() {
        String::new()
    } else {
        sections.join("\n\n")
    }
}

fn truncate_memory(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max).collect();
    out.push_str("…");
    out
}

/// Sections skills pour enrichir le contexte agent.
#[must_use]
pub fn skill_sections(
    config: &AgentConfig,
    skills: Option<&SkillRegistry>,
    user_message: &str,
) -> String {
    if !config.skill_auto_suggest {
        return String::new();
    }
    let Some(registry) = skills else {
        return String::new();
    };
    let mut sections = Vec::new();
    let catalog = format_skill_catalog(registry);
    if !catalog.is_empty() {
        sections.push(format!("## Skills disponibles\n{catalog}"));
    }
    let suggested = suggest_skills(&registry.list(), user_message, 3);
    if !suggested.is_empty() {
        let lines: Vec<String> = suggested
            .iter()
            .map(|s| format!("- {} — {}", s.name, s.description))
            .collect();
        sections.push(format!(
            "## Skills suggérées pour ce message\n{}",
            lines.join("\n")
        ));
    }
    sections.join("\n\n")
}

fn format_skill_catalog(registry: &SkillRegistry) -> String {
    registry
        .list()
        .into_iter()
        .map(|entry| format!("- {} — {}", entry.name, entry.description))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Formate les définitions d'outils pour le prompt système.
#[must_use]
pub fn format_tool_definitions(tools: &ToolRegistry) -> String {
    let mut lines = Vec::new();
    for def in tools.definitions() {
        lines.push(format!(
            "- **{}**: {}\n  Paramètres: {}",
            def.name, def.description, def.parameters_schema
        ));
    }
    lines.join("\n")
}

/// Prompt système de base pour la boucle agent.
#[must_use]
#[allow(dead_code)]
pub fn base_system_prompt(tool_section: &str, context_section: &str) -> String {
    base_system_prompt_with_personality(None, tool_section, context_section)
}

/// Prompt système avec personnalité agent persistant optionnelle (Phase 2b).
#[must_use]
pub fn base_system_prompt_with_personality(
    personality: Option<&str>,
    tool_section: &str,
    context_section: &str,
) -> String {
    let mut prompt = String::new();
    if let Some(p) = personality.filter(|s| !s.trim().is_empty()) {
        prompt.push_str("## Personnalité agent\n");
        prompt.push_str(p.trim());
        prompt.push_str("\n\n");
    } else {
        prompt.push_str(
            "Tu es l'agent Orchestrateur — second cerveau souverain avec accès au Cortex (mémoires persistantes).\n",
        );
    }
    prompt.push_str(
        "Utilise les outils via un bloc JSON exactement ainsi :\n\
         ```tool\n{\"name\":\"NOM_OUTIL\",\"arguments\":{...}}\n```\n\
         Tu peux enchaîner plusieurs outils. Quand tu as assez d'informations, réponds en français sans bloc tool.\n\n\
         ## Outils disponibles\n",
    );
    prompt.push_str(tool_section);
    if !context_section.is_empty() {
        prompt.push_str("\n\n## Contexte Cortex\n");
        prompt.push_str(context_section);
    }
    prompt
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::adapters::CortexContextProvider;
    use crate::testing::MockBundle;
    use cortex::ContextProvider;

    #[tokio::test]
    async fn context_provider_returns_agent_context() {
        let deps = MockBundle::new().into_deps();
        let config = AgentConfig::default();
        let provider = CortexContextProvider::new(deps, config);
        let ctx = provider
            .build_context("hello", None, 5)
            .await
            .expect("contexte agent");
        assert!(ctx.session_turns.is_empty());
    }
}