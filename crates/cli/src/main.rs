//! CLI Orchestrateur — commandes claires, update simple, bonne UX par défaut.

mod cli;
mod commands;
mod context;
mod dispatch;
mod present;
mod output;
mod tui;
mod windows_ops;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("orchestrateur=info".parse()?))
        .init();

    let cli = Cli::parse();
    dispatch::run(cli).await
}