use super::registry::ToolRegistry;

/// Profil de capacités agent — groupe d'outils Cortex (Phase 10+).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityProfileDescriptor {
    /// Identifiant TOML / CLI (`memory`, `agent`, `full`, …).
    pub id: &'static str,
    /// Nom affiché.
    pub display_name: &'static str,
    /// Outils inclus.
    pub tools: &'static [&'static str],
}

/// Catalogue des profils de capacités.
pub const CAPABILITY_PROFILE_DESCRIPTORS: &[CapabilityProfileDescriptor] = &[
    CapabilityProfileDescriptor {
        id: "memory",
        display_name: "Mémoire Cortex",
        tools: &["memory_search", "memory_get", "memory_assimilate", "memory_file_context"],
    },
    CapabilityProfileDescriptor {
        id: "mcp",
        display_name: "MCP distant",
        tools: &["mcp_list_tools", "mcp_call"],
    },
    CapabilityProfileDescriptor {
        id: "agent",
        display_name: "Agent standard",
        tools: &[
            "memory_search",
            "memory_get",
            "memory_assimilate",
            "memory_file_context",
            "mcp_list_tools",
            "mcp_call",
            "skill_list",
            "skill_execute",
            "skill_suggest",
            "skills_list",
            "skill_view",
            "skill_manage",
            "session_search",
            "todo",
            "memory",
            "read_file",
            "clarify",
        ],
    },
    CapabilityProfileDescriptor {
        id: "skills",
        display_name: "Skills agentic",
        tools: &["skill_list", "skill_execute", "skill_suggest"],
    },
    CapabilityProfileDescriptor {
        id: "research",
        display_name: "Recherche mémoire",
        tools: &["memory_search", "memory_get"],
    },
    CapabilityProfileDescriptor {
        id: "ingest",
        display_name: "Assimilation",
        tools: &["memory_assimilate", "memory_search"],
    },
    CapabilityProfileDescriptor {
        id: "full",
        display_name: "Complet (tous outils enregistrés)",
        tools: &[],
    },
];

/// Registre des profils de capacités.
#[derive(Debug, Clone, Copy, Default)]
pub struct CapabilityProfileRegistry;

impl CapabilityProfileRegistry {
    /// Nouveau registre (catalogue statique).
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Tous les descripteurs.
    #[must_use]
    pub fn descriptors(&self) -> &'static [CapabilityProfileDescriptor] {
        CAPABILITY_PROFILE_DESCRIPTORS
    }

    /// Recherche un profil par identifiant.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&'static CapabilityProfileDescriptor> {
        CAPABILITY_PROFILE_DESCRIPTORS.iter().find(|p| p.id == id)
    }

    /// Filtre un registre d'outils selon le profil actif.
    #[must_use]
    pub fn filter_registry(source: &ToolRegistry, profile_id: &str) -> ToolRegistry {
        let Some(descriptor) = Self::new().get(profile_id) else {
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
    fn catalog_has_seven_capability_profiles() {
        assert_eq!(CapabilityProfileRegistry::new().descriptors().len(), 7);
    }

    #[test]
    fn research_profile_filters_memory_tools() {
        let base = ToolRegistry::with_memory_tools();
        let filtered = CapabilityProfileRegistry::filter_registry(&base, "research");
        let names = filtered.names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"memory_search"));
        assert!(names.contains(&"memory_get"));
        assert!(!names.contains(&"memory_assimilate"));
    }
}