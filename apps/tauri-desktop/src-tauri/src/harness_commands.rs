//! Commandes Tauri harness — onboard, canaux, sondes (style Hermes).

use std::path::{Path, PathBuf};
use std::process::Command as OsCommand;

use orchestrator::gateway::resolve_channel_config;
use orchestrator::{
    set_channel_enabled, set_primary_llm, set_security_profile, ChannelCatalog, OrchestratorConfig,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::territory_launcher::repo_root_for_harness;

const DAEMON_TASK: &str = "OrchestrateurDaemon";
const DAEMON_TOKEN_ENV: &str = "ORCHESTRATEUR_DAEMON_TOKEN";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct HarnessWorkspaceInfo {
    pub path: String,
    pub config_exists: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct HarnessServiceProbe {
    pub daemon: String,
    pub gateway: String,
    pub daemon_url: String,
    pub gateway_url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ChannelRow {
    pub id: String,
    pub display_name: String,
    pub enabled: bool,
    pub token_env: String,
    pub token_set: bool,
    pub dedicated: bool,
    pub badges: Vec<String>,
    pub setup_url: String,
    pub setup_hint: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OnboardRequest {
    pub profile: String,
    pub llm: String,
    pub install_daemon: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SaveChannelRequest {
    pub channel_id: String,
    pub token: Option<String>,
    pub allowed_ids: Option<String>,
    pub enabled: bool,
}

fn harness_workspace() -> Result<PathBuf, String> {
    if let Ok(p) = std::env::var("ORCHESTRATEUR_HARNESS_WORKSPACE") {
        return Ok(PathBuf::from(p));
    }
    if let Some(root) = repo_root_for_harness() {
        let ws = root.join("workspace");
        if ws.join("config").exists() || root.join("workspace").exists() {
            return Ok(ws);
        }
    }
    Ok(PathBuf::from("workspace"))
}

fn config_path(workspace: &Path) -> PathBuf {
    workspace.join("config").join("orchestrator.toml")
}

async fn probe_url(client: &Client, url: &str) -> String {
    match client.get(url).send().await {
        Ok(r) if r.status().is_success() => "alive".into(),
        Ok(_) => "down".into(),
        Err(_) => "down".into(),
    }
}

fn channel_meta(id: &str) -> (&'static str, &'static str) {
    match id {
        "telegram" => (
            "https://core.telegram.org/bots/tutorial",
            "BotFather → créer un bot → copier le token",
        ),
        "discord" => (
            "https://discord.com/developers/applications",
            "Developer Portal → Bot → token → inviter sur le serveur",
        ),
        "slack" => (
            "https://api.slack.com/apps",
            "Créer une app Slack → Bot Token → installer sur le workspace",
        ),
        _ => ("", "Configurer les credentials du canal"),
    }
}

fn allowed_env(channel_id: &str) -> Option<&'static str> {
    match channel_id {
        "discord" => Some("ORCHESTRATEUR_DISCORD_ALLOWED_USER_IDS"),
        "telegram" => Some("ORCHESTRATEUR_TELEGRAM_ALLOWED_USER_IDS"),
        _ => None,
    }
}

fn set_user_env(name: &str, value: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        let escaped = value.replace('\'', "''");
        OsCommand::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "[Environment]::SetEnvironmentVariable('{name}', '{escaped}', 'User')"
                ),
            ])
            .status()
            .map_err(|e| e.to_string())?;
        std::env::set_var(name, value);
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        std::env::set_var(name, value);
        Ok(())
    }
}

fn ensure_workspace_tree(workspace: &Path) -> Result<(), String> {
    for dir in [
        workspace,
        &workspace.join("memories"),
        &workspace.join("logs"),
        &workspace.join("config"),
        &workspace.join(".orchestrateur").join("sessions"),
        &workspace.join(".orchestrateur").join("drafts"),
    ] {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn write_minimal_config(path: &Path) -> Result<(), String> {
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

[gateway.telegram]
enabled = false
token_env = "TELEGRAM_BOT_TOKEN"

[gateway.discord]
enabled = false
token_env = "DISCORD_BOT_TOKEN"

[watcher]
enabled = true
watch_dirs = [".orchestrateur/sessions"]
"#;
    std::fs::write(path, body).map_err(|e| e.to_string())
}

fn ensure_daemon_token() -> Result<(), String> {
    if std::env::var(DAEMON_TOKEN_ENV).is_ok() {
        return Ok(());
    }
    #[cfg(windows)]
    {
        let output = OsCommand::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "[Environment]::GetEnvironmentVariable('{DAEMON_TOKEN_ENV}', 'User')"
                ),
            ])
            .output()
            .map_err(|e| e.to_string())?;
        let existing = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !existing.is_empty() {
            std::env::set_var(DAEMON_TOKEN_ENV, &existing);
            return Ok(());
        }
        let token = uuid::Uuid::new_v4().as_simple().to_string();
        set_user_env(DAEMON_TOKEN_ENV, &token)?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let token = uuid::Uuid::new_v4().as_simple().to_string();
        std::env::set_var(DAEMON_TOKEN_ENV, &token);
        Ok(())
    }
}

#[tauri::command]
pub fn harness_workspace_info() -> Result<HarnessWorkspaceInfo, String> {
    let path = harness_workspace()?;
    let cfg = config_path(&path);
    Ok(HarnessWorkspaceInfo {
        path: path.display().to_string(),
        config_exists: cfg.is_file(),
    })
}

#[tauri::command]
pub async fn harness_probe_services() -> Result<HarnessServiceProbe, String> {
    let workspace = harness_workspace()?;
    let config = OrchestratorConfig::load_workspace(&workspace).map_err(|e| e.to_string())?;
    let daemon_url = format!(
        "http://{}:{}/health",
        config.daemon.bind, config.daemon.port
    );
    let gateway_url = format!(
        "http://{}:{}/health",
        config.gateway.bind, config.gateway.port
    );
    let client = Client::builder()
        .user_agent("Orchestrateur-Desktop")
        .build()
        .map_err(|e| e.to_string())?;
    let daemon = probe_url(&client, &daemon_url).await;
    let gateway = probe_url(&client, &gateway_url).await;
    Ok(HarnessServiceProbe {
        daemon,
        gateway,
        daemon_url,
        gateway_url,
    })
}

#[tauri::command]
pub fn harness_list_channels() -> Result<Vec<ChannelRow>, String> {
    let workspace = harness_workspace()?;
    let config = OrchestratorConfig::load_workspace(&workspace).map_err(|e| e.to_string())?;
    let catalog = ChannelCatalog::new();
    let mut rows = Vec::new();
    for d in catalog.descriptors() {
        if !d.dedicated || !matches!(d.id, "telegram" | "discord" | "slack") {
            continue;
        }
        let cfg = resolve_channel_config(&config.gateway, d.id);
        let token_set = !cfg.token_env.is_empty() && std::env::var(&cfg.token_env).is_ok();
        let mut badges = Vec::new();
        if cfg.enabled {
            badges.push("actif".into());
        } else {
            badges.push("désactivé".into());
        }
        if !token_set {
            badges.push("needs setup".into());
        }
        let (setup_url, setup_hint) = channel_meta(d.id);
        rows.push(ChannelRow {
            id: d.id.to_string(),
            display_name: d.display_name.to_string(),
            enabled: cfg.enabled,
            token_env: cfg.token_env.clone(),
            token_set,
            dedicated: d.dedicated,
            badges,
            setup_url: setup_url.to_string(),
            setup_hint: setup_hint.to_string(),
        });
    }
    Ok(rows)
}

#[tauri::command]
pub fn harness_save_channel(req: SaveChannelRequest) -> Result<(), String> {
    let workspace = harness_workspace()?;
    let settings = config_path(&workspace);
    if !settings.is_file() {
        return Err("config absente — terminez l'onboard d'abord".into());
    }
    if let Some(token) = req.token.as_deref() {
        let trimmed = token.trim();
        if !trimmed.is_empty() {
            let cfg = OrchestratorConfig::load_workspace(&workspace).map_err(|e| e.to_string())?;
            let ch = resolve_channel_config(&cfg.gateway, &req.channel_id);
            if !ch.token_env.is_empty() {
                set_user_env(&ch.token_env, trimmed)?;
            }
        }
    }
    if let Some(ids) = req.allowed_ids.as_deref() {
        if let Some(env) = allowed_env(&req.channel_id) {
            let trimmed = ids.trim();
            if !trimmed.is_empty() {
                set_user_env(env, trimmed)?;
            }
        }
    }
    if req.enabled {
        set_channel_enabled(&settings, &req.channel_id, true).map_err(|e| e.to_string())?;
    } else {
        set_channel_enabled(&settings, &req.channel_id, false).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn harness_apply_onboard(req: OnboardRequest) -> Result<(), String> {
    let workspace = harness_workspace()?;
    ensure_workspace_tree(&workspace)?;
    let settings = config_path(&workspace);
    if !settings.is_file() {
        let example = workspace
            .parent()
            .map(|p| p.join("workspace/config/orchestrator.toml.example"))
            .filter(|p| p.is_file());
        if let Some(src) = example {
            std::fs::copy(&src, &settings).map_err(|e| e.to_string())?;
        } else {
            write_minimal_config(&settings)?;
        }
    }
    set_security_profile(&settings, &req.profile).map_err(|e| e.to_string())?;
    set_primary_llm(&settings, &req.llm).map_err(|e| e.to_string())?;
    ensure_daemon_token()?;
    if req.install_daemon {
        install_daemon_task(&workspace)?;
    }
    Ok(())
}

fn install_daemon_task(workspace: &Path) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let ws = workspace.display();
    let args = format!("daemon run --workspace \"{ws}\"");
    #[cfg(windows)]
    {
        let tr = format!("\"{}\" {}", exe.display(), args);
        let status = OsCommand::new("schtasks")
            .args([
                "/Create",
                "/F",
                "/TN",
                DAEMON_TASK,
                "/TR",
                &tr,
                "/SC",
                "ONLOGON",
                "/RL",
                "LIMITED",
            ])
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err("échec création tâche planifiée daemon".into())
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (workspace, exe, args);
        Ok(())
    }
}