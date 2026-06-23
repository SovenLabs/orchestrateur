//! `orch health` — sonde rapide du bridge.

use anyhow::Result;
use orchestrator::{Command, OrchestratorFacade};

use crate::context::run_bridge_command;

pub async fn run(facade: &OrchestratorFacade) -> Result<()> {
    run_bridge_command(facade, Command::HealthCheck).await
}