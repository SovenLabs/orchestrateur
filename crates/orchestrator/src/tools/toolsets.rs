use super::registry::ToolRegistry;

/// Groupe nommé d'outils agent (style Hermess toolsets).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolsetDescriptor {
    /// Identifiant TOML / CLI (`memory`, `agent`, `full`, …).
    pub id: &'static str,
    /// Nom affiché.
    pub display_name: &'static str,
    /// Outils inclus.
    pub tools: &'static [&'static str],
}

/// Catalogue des toolsets Phase 10.
pub const TOOLSET_DESCRIPTORS: &[ToolsetDescriptor] = &[
    ToolsetDescriptor {
        id: "memory",
        display_name: "Mémoire Cortex",
        tools: &["memory_search", "memory_get", "memory_assimilate"],
    },
    ToolsetDescriptor {
        id: "mcp",
        display_name: "MCP distant",
        tools: &["mcp_list_tools", "mcp_call"],
    },
    ToolsetDescriptor {
        id: "agent",
        display_name: "Agent standard",
        tools: &[
            "memory_search",
            "memory_get",
            "memory_assimilate",
            "mcp_list_tools",
            "mcp_call",
            "skill_list",
            "skill_execute",
            "skill_suggest",
        ],
    },
    ToolsetDescriptor {
        id: "skills",
        display_name: "Skills agentic",
        tools: &["skill_list", "skill_execute", "skill_suggest"],
    },
    ToolsetDescriptor {
        id: "research",
        display_name: "Recherche mémoire",
        tools: &["memory_search", "memory_get"],
    },
    ToolsetDescriptor {
        id: "ingest",
        display_name: "Assimilation",
        tools: &["memory_assimilate", "memory_search"],
    },
    ToolsetDescriptor {
        id: "full",
        display_name: "Complet (tous outils enregistrés)",
        tools: &[],
    },
];

/// Registre des toolsets.
#[derive(Debug, Clone, Copy, Default)]
pub struct ToolsetRegistry;

impl ToolsetRegistry {
    /// Nouveau registre (catalogue statique).
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Tous les descripteurs.
    #[must_use]
    pub fn descriptors(&self) -> &'static [ToolsetDescriptor] {
        TOOLSET_DESCRIPTORS
    }

    /// Recherche un toolset par identifiant.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&'static ToolsetDescriptor> {
        TOOLSET_DESCRIPTORS.iter().find(|t| t.id == id)
    }

    /// Filtre un registre d'outils selon le toolset actif.
    #[must_use]
    pub fn filter_registry(source: &ToolRegistry, toolset_id: &str) -> ToolRegistry {
        let Some(descriptor) = Self::new().get(toolset_id) else {
            return source.clone_subset(source.names().as_slice());
        };
        if descriptor.id == "full" {
            return source.clone_subset(source.names().as_slice());
        }
        source.clone_subset(descriptor.tools)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_seven_toolsets() {
        assert_eq!(ToolsetRegistry::new().descriptors().len(), 7);
    }

    #[test]
    fn research_toolset_filters_memory_tools() {
        let base = ToolRegistry::with_memory_tools();
        let filtered = ToolsetRegistry::filter_registry(&base, "research");
        let names = filtered.names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"memory_search"));
        assert!(names.contains(&"memory_get"));
        assert!(!names.contains(&"memory_assimilate"));
    }
}