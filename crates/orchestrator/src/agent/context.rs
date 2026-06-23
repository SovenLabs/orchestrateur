use cortex::{KnowledgeGraph, Memory, SearchFilter};
use crate::deps::AppDependencies;
use crate::skills::suggest_skills;
use crate::skills::SkillRegistry;
use crate::tools::{ToolContext, ToolRegistry};
use crate::use_cases::ListMemories;

use super::config::AgentConfig;
use super::AgentError;

/// Contexte enrichi injecté dans le prompt système.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltContext {
    /// Bloc texte pour le prompt système.
    pub system_context: String,
    /// Résultat de la recherche proactive (si exécutée).
    pub proactive_search: Option<String>,
}

/// Construit le contexte graphe + recherche proactive.
pub async fn build_context(
    deps: &AppDependencies,
    tools: &ToolRegistry,
    config: &AgentConfig,
    user_message: &str,
    skills: Option<&SkillRegistry>,
) -> Result<BuiltContext, AgentError> {
    let mut sections = Vec::new();

    if config.graph_context_enabled {
        if let Some(graph_section) = build_graph_section(deps, config.graph_hub_limit).await? {
            sections.push(graph_section);
        }
    }

    let mut proactive_search = None;
    if config.proactive_memory_search {
        let ctx = ToolContext::new(deps.clone());
        let args = serde_json::json!({
            "query": user_message,
            "limit": config.proactive_search_limit
        });
        match tools.execute(&ctx, "memory_search", &args).await {
            Ok(result) => {
                proactive_search = Some(result.content.clone());
                sections.push(format!(
                    "## Souvenirs pertinents (recherche proactive)\n{}",
                    result.content
                ));
            }
            Err(err) => {
                sections.push(format!(
                    "## Souvenirs pertinents\n(recherche indisponible: {err})"
                ));
            }
        }
    }

    if config.skill_auto_suggest {
        if let Some(registry) = skills {
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
        }
    }

    let system_context = if sections.is_empty() {
        String::new()
    } else {
        sections.join("\n\n")
    };

    Ok(BuiltContext {
        system_context,
        proactive_search,
    })
}

async fn build_graph_section(
    deps: &AppDependencies,
    hub_limit: usize,
) -> Result<Option<String>, AgentError> {
    let memories = ListMemories::new(deps.clone()).execute().await?;
    if memories.is_empty() {
        return Ok(None);
    }

    let graph = KnowledgeGraph::from_memories(&memories);
    let title_by_id: std::collections::HashMap<_, _> = memories
        .iter()
        .map(|m: &Memory| (m.id, m.title.as_str()))
        .collect();

    let hubs = graph
        .hub_ranking()
        .into_iter()
        .take(hub_limit)
        .map(|(id, inbound)| {
            let title = title_by_id
                .get(&id)
                .map_or_else(|| id.to_string(), |t| (*t).to_string());
            format!("- {title} ({inbound} liens entrants)")
        })
        .collect::<Vec<_>>();

    Ok(Some(format!(
        "## Graphe de connaissances\nNœuds: {}, arêtes: {}\n\n### Hubs\n{}",
        graph.node_count(),
        graph.edge_count(),
        hubs.join("\n")
    )))
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
pub fn base_system_prompt(tool_section: &str, context_section: &str) -> String {
    let mut prompt = String::from(
        "Tu es l'agent Orchestrateur — second cerveau souverain avec accès au Cortex (mémoires persistantes).\n\
         Utilise les outils via un bloc JSON exactement ainsi :\n\
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
    use crate::testing::MockBundle;

    #[tokio::test]
    async fn build_context_empty_graph() {
        let deps = MockBundle::new().into_deps();
        let tools = ToolRegistry::with_memory_tools();
        let config = AgentConfig::default();
        let ctx = build_context(&deps, &tools, &config, "hello", None)
            .await
            .unwrap();
        assert!(ctx.system_context.contains("recherche") || ctx.proactive_search.is_some());
    }
}