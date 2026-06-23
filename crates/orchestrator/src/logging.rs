//! Initialisation du logging structuré (stdout + fichier workspace).

use std::fs::{self, OpenOptions};
use std::path::Path;
use std::sync::OnceLock;

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static LOG_INIT: OnceLock<()> = OnceLock::new();

/// Options d'initialisation du logging applicatif.
#[derive(Debug, Clone)]
pub struct LoggingOptions {
    /// Filtre `tracing` (ex. `orchestrateur=info`).
    pub filter: String,
    /// Écrit aussi dans `workspace/logs/orchestrateur.log` si `true`.
    pub file_appender: bool,
}

impl Default for LoggingOptions {
    fn default() -> Self {
        Self {
            filter: "orchestrateur=info,cortex=warn,infrastructure=warn".into(),
            file_appender: true,
        }
    }
}

/// Initialise le subscriber global une seule fois.
///
/// # Panics
///
/// Panique si le filtre env est invalide ou si le fichier log est inaccessible.
pub fn init_logging(workspace_root: &Path, options: &LoggingOptions) {
    LOG_INIT.get_or_init(|| {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(options.filter.clone()));

        let stdout_layer = fmt::layer().with_target(true);

        if options.file_appender {
            let log_dir = workspace_root.join("logs");
            fs::create_dir_all(&log_dir).expect("création logs/");
            let log_path = log_dir.join("orchestrateur.log");
            let file_layer = fmt::layer()
                .with_target(true)
                .with_ansi(false)
                .with_writer(move || {
                    OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&log_path)
                        .expect("ouverture orchestrateur.log")
                });
            tracing_subscriber::registry()
                .with(filter)
                .with(stdout_layer)
                .with(file_layer)
                .init();
        } else {
            tracing_subscriber::registry()
                .with(filter)
                .with(stdout_layer)
                .init();
        }
    });
}