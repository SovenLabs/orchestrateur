//! Arborescence workspace harness.

use std::path::{Path, PathBuf};

use crate::harness::error::HarnessError;

/// Crée l'arborescence workspace standard.
pub fn ensure_workspace_tree(workspace: &Path) -> Result<(), HarnessError> {
    for dir in [
        workspace,
        &workspace.join("memories"),
        &workspace.join("logs"),
        &workspace.join("config"),
        &workspace.join(".orchestrateur").join("sessions"),
        &workspace.join(".orchestrateur").join("drafts"),
    ] {
        std::fs::create_dir_all(dir).map_err(|e| HarnessError::io(dir, e.to_string()))?;
    }
    Ok(())
}

/// Chemin `orchestrator.toml` dans le workspace.
#[must_use]
pub fn config_path(workspace: &Path) -> PathBuf {
    workspace.join("config").join("orchestrator.toml")
}

/// Cherche un fichier exemple de configuration.
#[must_use]
pub fn find_example_config() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("workspace/config/orchestrator.toml.example"),
        PathBuf::from("config/orchestrator.toml.example"),
    ];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            for alias in ["orchestrator.toml.example", "workspace/config/orchestrator.toml.example"]
            {
                let p = parent.join(alias);
                if p.exists() {
                    return Some(p);
                }
            }
        }
    }
    for c in candidates {
        if c.exists() {
            return Some(c);
        }
    }
    None
}

/// Écrit une configuration minimale souveraine.
pub fn write_minimal_config(path: &Path) -> Result<(), HarnessError> {
    let body = r#"[providers]
primary_llm = "ollama"
fallback_llm = []
primary_embedding = "ollama"

[security]
profile = "local_only"

[daemon]
enabled = true
bind = "127.0.0.1"
port = 28790
token_env = "ORCHESTRATEUR_DAEMON_TOKEN"

[gateway]
enabled = true
bind = "127.0.0.1"
port = 28789
token_env = "ORCHESTRATEUR_GATEWAY_TOKEN"

[watcher]
enabled = true
watch_dirs = [".orchestrateur/sessions"]
"#;
    std::fs::write(path, body).map_err(|e| HarnessError::io(path, e.to_string()))?;
    Ok(())
}