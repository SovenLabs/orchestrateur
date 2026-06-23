//! Contexte partagé CLI — bootstrap workspace et exécution bridge.

use std::path::Path;

use anyhow::Result;
use infrastructure::bootstrap_workspace;
use orchestrator::{execute_command, Command, OrchestratorFacade};

use crate::output::print_response;

/// Bootstrap le facade Cortex depuis le workspace.
pub async fn bootstrap_facade(workspace: &Path) -> Result<OrchestratorFacade> {
    let deps = bootstrap_workspace(workspace)
        .await
        .map_err(|err| anyhow::anyhow!(err.with_context("CLI")))?;
    Ok(OrchestratorFacade::new(deps))
}

/// Exécute une commande bridge et affiche la réponse.
pub async fn run_bridge_command(facade: &OrchestratorFacade, command: Command) -> Result<()> {
    let response = execute_command(facade, command).await;
    print_response(response)
}

