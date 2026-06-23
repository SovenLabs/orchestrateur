//! `orch doctor` — diagnostic structuré.

use std::path::Path;

use anyhow::Result;
use orchestrator::OrchestratorFacade;

use crate::harness_ops::cmd_doctor;

/// Diagnostic complet (délègue à harness_ops, sections structurées).
pub async fn run(facade: &OrchestratorFacade, workspace: &Path) -> Result<()> {
    cmd_doctor(facade, workspace).await
}