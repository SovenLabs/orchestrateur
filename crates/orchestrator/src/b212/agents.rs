//! Catalogue des 6 agents domaine B212 (MoltX / Stratos).

/// Définition d'un agent domaine B212.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct B212AgentDef {
    /// Identifiant persistant (`b212-*`).
    pub id: &'static str,
    /// Nom affiché.
    pub name: &'static str,
    /// Rôle fonctionnel orchestrateur.
    pub role: &'static str,
    /// Rôle Bible / MoltX.
    pub bible_role: &'static str,
    /// Ordre dans le workflow desk.
    pub order: u8,
}

/// Les six agents domaine B212 dans l'ordre canonique du workflow.
pub const B212_AGENTS: [B212AgentDef; 6] = [
    B212AgentDef {
        id: "b212-research-analyst",
        name: "Orbital Eye",
        role: "b212_macro",
        bible_role: "Macro Liquidity Sentinel",
        order: 1,
    },
    B212AgentDef {
        id: "b212-market-regime",
        name: "Iron Map",
        role: "b212_regime",
        bible_role: "Market Structure Commander — Regime",
        order: 2,
    },
    B212AgentDef {
        id: "b212-structure",
        name: "Iron Map Structure",
        role: "b212_structure",
        bible_role: "Market Structure Commander — Structure",
        order: 3,
    },
    B212AgentDef {
        id: "b212-order-flow",
        name: "Shadow Sweep",
        role: "b212_orderflow",
        bible_role: "Liquidity & Order Flow Hunter",
        order: 4,
    },
    B212AgentDef {
        id: "b212-risk-manager",
        name: "War Coordinator",
        role: "b212_risk",
        bible_role: "Risk report aggregator",
        order: 5,
    },
    B212AgentDef {
        id: "b212-execution",
        name: "Final Authority",
        role: "b212_execution",
        bible_role: "Execution General",
        order: 6,
    },
];

/// Retourne la définition d'un agent par identifiant.
#[must_use]
pub fn agent_def(id: &str) -> Option<&'static B212AgentDef> {
    B212_AGENTS.iter().find(|a| a.id == id)
}