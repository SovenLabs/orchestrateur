//! Binaire TUI ratatui — interface terminal interactive (séparé du CLI headless).

use std::io::IsTerminal;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use infrastructure::bootstrap_workspace;
use orchestrator::TuiApp;
use orchestrator_client::OrchestratorClient;
use tracing_subscriber::EnvFilter;

/// Orchestrateur — interface terminal (ratatui).
#[derive(Parser)]
#[command(
    name = "orchestrateur-tui",
    version,
    about = "Orchestrateur v0.6.0 — interface terminal (TUI)"
)]
struct Cli {
    /// Racine du workspace (défaut: ./workspace).
    #[arg(long, default_value = "workspace")]
    workspace: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("orchestrateur=info".parse()?))
        .init();

    if !std::io::stdin().is_terminal() {
        anyhow::bail!("TUI requiert un terminal interactif (stdin n'est pas un TTY)");
    }

    let cli = Cli::parse();
    let deps = bootstrap_workspace(&cli.workspace)
        .await
        .map_err(|err| anyhow::anyhow!(err.with_context("TUI")))?;

    let mut client = OrchestratorClient::connect(deps)
        .map_err(|err| anyhow::anyhow!("démarrage bridge orchestrateur: {err}"))?;

    let handle = client.handle().clone();
    let thread = client
        .take_thread()
        .context("thread orchestrateur déjà consommé")?;
    let mut app = TuiApp::new(handle, thread);
    let run_result = tokio::task::spawn_blocking(move || app.run()).await;

    match run_result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(err)) => Err(anyhow::anyhow!("TUI: {err}")),
        Err(join_err) => Err(anyhow::anyhow!("TUI thread: {join_err}")),
    }
}