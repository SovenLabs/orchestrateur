//! Binaire HUD egui — composition racine (wiring infrastructure + bridge).

mod app;
mod icon;
mod list;
mod metrics;
mod prefs;
mod search_list;
mod state;
mod theme;

use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use eframe::egui;
use infrastructure::{build_app_dependencies, WiringError};
use orchestrator::{spawn_orchestrator_bridge, OrchestratorConfig};
use tracing_subscriber::EnvFilter;

use app::HudApp;
use icon::app_icon;

/// Orchestrateur HUD — interface native egui.
#[derive(Parser)]
#[command(name = "orchestrateur-hud", version, about = "Orchestrateur HUD egui — Phase 4")]
struct Cli {
    /// Racine du workspace (défaut: ./workspace).
    #[arg(long, default_value = "workspace")]
    workspace: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("orchestrateur=info".parse()?))
        .init();

    let cli = Cli::parse();
    let config = OrchestratorConfig::load_workspace(&cli.workspace)
        .with_context(|| format!("chargement config depuis {}", cli.workspace.display()))?;

    let deps = match build_app_dependencies(config).await {
        Ok(deps) => deps,
        Err(WiringError::MemoryMode) => {
            anyhow::bail!(
                "vector_store type=memory : configurez type=lancedb dans orchestrator.toml pour le HUD"
            );
        }
        Err(err) => return Err(err.into()),
    };

    let (handle, thread) = spawn_orchestrator_bridge(deps)
        .map_err(|err| anyhow::anyhow!("démarrage bridge orchestrateur: {err}"))?;

    let icon = app_icon();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_icon(icon),
        persist_window: true,
        ..Default::default()
    };

    let run_result = eframe::run_native(
        "Orchestrateur",
        options,
        Box::new(move |cc| Ok(Box::new(HudApp::new(cc, handle, thread)))),
    );

    if let Err(err) = run_result {
        anyhow::bail!("eframe: {err}");
    }
    Ok(())
}