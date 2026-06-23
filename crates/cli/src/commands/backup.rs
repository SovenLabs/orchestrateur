//! `orch backup` — sauvegarde et restauration du workspace.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Subcommand;
use console::style;
use orchestrator::backup::{create_backup, restore_backup};
use orchestrator::OrchestratorConfig;

/// Sous-commandes backup.
#[derive(Subcommand, Clone, Debug)]
pub enum BackupCommands {
    /// Crée une sauvegarde horodatée du workspace.
    Create {
        /// Dossier parent recevant `orchestrateur-backup-<timestamp>/`.
        #[arg(long)]
        dest: Option<PathBuf>,
    },
    /// Restaure un workspace depuis un dossier de sauvegarde.
    Restore {
        /// Chemin du dossier `orchestrateur-backup-*`.
        backup_dir: PathBuf,
    },
    /// Liste les chemins inclus dans une sauvegarde par défaut.
    Plan,
}

/// Exécute une commande backup.
pub fn run(command: BackupCommands, workspace: &Path) -> Result<()> {
    let config = OrchestratorConfig::load_workspace(workspace)
        .with_context(|| format!("chargement config workspace {}", workspace.display()))?;

    match command {
        BackupCommands::Create { dest } => {
            let parent = dest
                .unwrap_or_else(|| workspace.join("backups"));
            std::fs::create_dir_all(&parent)?;
            let report = create_backup(&config, &parent)?;
            println!(
                "{} Sauvegarde créée — {} fichiers",
                style("✓").green().bold(),
                report.files_copied
            );
            println!("  {}", style(report.backup_dir.display()).cyan());
            println!(
                "  version {} · {} chemins",
                report.manifest.orchestrator_version,
                report.manifest.paths.len()
            );
        }
        BackupCommands::Restore { backup_dir } => {
            let manifest = restore_backup(&backup_dir, workspace)?;
            println!(
                "{} Workspace restauré depuis {}",
                style("✓").green().bold(),
                backup_dir.display()
            );
            println!(
                "  backup {} · {} chemins",
                manifest.created_at,
                manifest.paths.len()
            );
        }
        BackupCommands::Plan => {
            let paths = orchestrator::backup::default_backup_paths(&config);
            println!("{}", style("Plan de sauvegarde par défaut").bold());
            for path in paths {
                let exists = workspace.join(&path).exists();
                let badge = if exists {
                    style("[présent]").green()
                } else {
                    style("[absent]").dim()
                };
                println!("  {} {}", path.display(), badge);
            }
        }
    }
    Ok(())
}