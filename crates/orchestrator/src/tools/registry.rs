use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

use crate::agent::AgentConfig;
use crate::deps::AppDependencies;
use crate::skills::SkillRegistry;
use super::capability_profiles::CapabilityProfileRegistry;
use super::memory_assimilate::MemoryAssimilateTool;
use super::memory_file_context::MemoryFileContextTool;
use super::memory_get::MemoryGetTool;
use super::memory_search::MemorySearchTool;
use super::tool::{Tool, ToolContext, ToolDefinition, ToolError, ToolResult};

/// Registre central des outils agent Phase 7.
pub struct ToolRegistry {
    tools: HashMap<&'static str, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Registre vide.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Registre avec les 3 outils mémoire par défaut.
    #[must_use]
    pub fn with_memory_tools() -> Self {
        let mut registry = Self::new();
        registry.register(Arc::new(MemorySearchTool));
        registry.register(Arc::new(MemoryGetTool));
        registry.register(Arc::new(MemoryAssimilateTool));
        registry.register(Arc::new(MemoryFileContextTool));
        registry
    }

    /// Registre mémoire + outils MCP Phase 9.
    #[must_use]
    pub fn with_memory_and_mcp_tools() -> Self {
        let mut registry = Self::with_memory_tools();
        registry.register(Arc::new(super::mcp_list::McpListToolsTool));
        registry.register(Arc::new(super::mcp_call::McpCallTool));
        registry
    }

    /// Construit le registre complet puis applique le profil de capacités Phase 10.
    #[must_use]
    pub fn build_for_deps(deps: &AppDependencies, profile_id: &str) -> Self {
        let base = if deps.mcp.is_some() {
            Self::with_memory_and_mcp_tools()
        } else {
            Self::with_memory_tools()
        };
        CapabilityProfileRegistry::filter_registry(&base, profile_id)
    }

    /// Construit le registre agent avec outils skills optionnels (Phase 12).
    #[must_use]
    pub fn build_for_agent(
        deps: &AppDependencies,
        config: &AgentConfig,
        skills: Option<Arc<SkillRegistry>>,
    ) -> Self {
        let mut registry = Self::build_for_deps(deps, &config.active_capability_profile);
        if config.skill_tools_enabled {
            if let Some(skills) = skills {
                registry.register(Arc::new(super::skill_list::SkillListTool::new(
                    Arc::clone(&skills),
                )));
                registry.register(Arc::new(super::skill_execute::SkillExecuteTool::new(
                    Arc::clone(&skills),
                )));
                registry.register(Arc::new(super::skill_suggest::SkillSuggestTool::new(
                    skills,
                )));
            }
        }
        registry
    }

    /// Clone un sous-ensemble d'outils par noms.
    #[must_use]
    pub fn clone_subset(&self, names: &[&str]) -> ToolRegistry {
        let mut filtered = ToolRegistry::new();
        for name in names {
            if let Some(tool) = self.tools.get(name) {
                filtered.register(Arc::clone(tool));
            }
        }
        filtered
    }

    /// Enregistre un outil (écrase si même nom).
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name(), tool);
    }

    /// Liste les définitions pour le prompt LLM.
    #[must_use]
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        let mut defs: Vec<_> = self.tools.values().map(|t| t.definition()).collect();
        defs.sort_by_key(|d| d.name);
        defs
    }

    /// Noms des outils enregistrés.
    #[must_use]
    pub fn names(&self) -> Vec<&'static str> {
        let mut names: Vec<_> = self.tools.keys().copied().collect();
        names.sort_unstable();
        names
    }

    /// Exécute un outil par nom.
    ///
    /// # Errors
    ///
    /// Retourne [`ToolError`] si l'outil est introuvable ou l'exécution échoue.
    pub async fn execute(
        &self,
        ctx: &ToolContext,
        name: &str,
        args: &Value,
    ) -> Result<ToolResult, ToolError> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| ToolError::NotFound(name.to_string()))?;
        tool.execute(ctx, args).await
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::with_memory_tools()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentConfig;
    use crate::testing::MockBundle;
    use std::sync::Arc;

    #[test]
    fn build_for_agent_includes_skill_tools_when_enabled() {
        let deps = MockBundle::new().into_deps();
        let skills = Arc::new(crate::skills::SkillRegistry::with_defaults());
        let config = AgentConfig::default();
        let registry = ToolRegistry::build_for_agent(&deps, &config, Some(skills));
        let names = registry.names();
        assert!(names.contains(&"skill_list"));
        assert!(names.contains(&"skill_execute"));
        assert!(names.contains(&"skill_suggest"));
    }

    #[tokio::test]
    async fn registry_lists_memory_tools() {
        let registry = ToolRegistry::with_memory_tools();
        let names = registry.names();
        assert!(names.contains(&"memory_search"));
        assert!(names.contains(&"memory_get"));
        assert!(names.contains(&"memory_assimilate"));
    }

    #[tokio::test]
    async fn memory_search_on_empty_repo() {
        let ctx = ToolContext::new(MockBundle::new().into_deps());
        let registry = ToolRegistry::with_memory_tools();
        let args = serde_json::json!({"query": "test"});
        let result = registry
            .execute(&ctx, "memory_search", &args)
            .await
            .unwrap();
        assert!(result.content.contains("Aucun"));
    }
}