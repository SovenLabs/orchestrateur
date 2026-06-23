//! `orch doctor` — diagnostic structuré.

use std::path::Path;

use anyhow::Result;
use orchestrator::OrchestratorFacade;

use crate::present;

pub async fn run(facade: &OrchestratorFacade, workspace: &Path) -> Result<()> {
    present::doctor(facade, workspace).await
}