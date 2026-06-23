//! Adapters orchestrator pour les ports Agent ↔ Cortex (`cortex::ports::agent_ports`).

mod assimilation_service;
mod change_detector;
mod context_provider;
mod semantic_search;

pub use assimilation_service::CortexAssimilationService;
pub use change_detector::{agent_exchange_turn, ChangeDetector, ChangeDetectorConfig};
pub use context_provider::CortexContextProvider;
pub use semantic_search::CortexSemanticSearch;

use std::sync::Arc;

use cortex::{AssimilationService, ContextProvider, SemanticSearch};

use crate::agent::AgentConfig;
use crate::deps::AppDependencies;

/// Fabrique les trois adapters agent à partir des dépendances applicatives.
#[must_use]
pub fn build_agent_adapters(
    deps: AppDependencies,
    config: AgentConfig,
) -> (
    Arc<dyn ContextProvider>,
    Arc<dyn AssimilationService>,
    Arc<dyn SemanticSearch>,
) {
    let semantic: Arc<dyn SemanticSearch> =
        Arc::new(CortexSemanticSearch::new(deps.clone()));
    let context: Arc<dyn ContextProvider> = Arc::new(CortexContextProvider::new(
        deps.clone(),
        config.clone(),
    ));
    let assimilation: Arc<dyn AssimilationService> =
        Arc::new(CortexAssimilationService::new(deps, config));
    (context, assimilation, semantic)
}