use std::sync::Arc;

use crate::cortex_extensions::CortexExtensionRegistry;
use crate::deps::AppDependencies;
use crate::registry::AgentRegistry;
use crate::skills::registry::SkillRegistry;

use super::skill::SkillContext;

/// Contexte hôte fourni aux skills typées (Cortex, Agent, B212, Communication).
#[derive(Clone)]
pub struct SkillHostContext {
    /// Dépendances injectées (ports Cortex, config, LLM…).
    pub deps: AppDependencies,
    /// Registre global des skills.
    pub skill_registry: Arc<SkillRegistry>,
    /// Points d'extension Cortex.
    pub cortex_extensions: Arc<CortexExtensionRegistry>,
    /// Registre agents (optionnel — injection par agent).
    pub agent_registry: Option<Arc<tokio::sync::RwLock<AgentRegistry>>>,
    /// Agent cible lors d'une exécution scoping agent.
    pub agent_id: Option<String>,
}

impl SkillHostContext {
    /// Construit un contexte global (sans agent ciblé).
    #[must_use]
    pub fn global(
        deps: AppDependencies,
        skill_registry: Arc<SkillRegistry>,
        cortex_extensions: Arc<CortexExtensionRegistry>,
    ) -> Self {
        Self {
            deps,
            skill_registry,
            cortex_extensions,
            agent_registry: None,
            agent_id: None,
        }
    }

    /// Attache un agent cible et le registre agents.
    #[must_use]
    pub fn for_agent(
        mut self,
        agent_id: impl Into<String>,
        agent_registry: Arc<tokio::sync::RwLock<AgentRegistry>>,
    ) -> Self {
        self.agent_id = Some(agent_id.into());
        self.agent_registry = Some(agent_registry);
        self
    }
}

/// Paramètres d'exécution + contexte hôte (Phase 6).
#[derive(Clone)]
pub struct SkillExecution {
    /// Paramètres d'entrée (query, text, tags…).
    pub input: SkillContext,
    /// Contexte hôte optionnel (skills typées).
    pub host: Option<SkillHostContext>,
}

impl SkillExecution {
    /// Exécution simple sans hôte.
    #[must_use]
    pub fn from_input(input: SkillContext) -> Self {
        Self { input, host: None }
    }
}