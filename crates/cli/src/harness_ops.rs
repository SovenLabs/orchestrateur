//! Opérations harness — onboard, doctor enrichi, daemon/gateway, canaux, providers.

use std::path::{Path, PathBuf};
use std::process::{Command as OsCommand, Stdio};
use std::time::Duration;

use anyhow::{Context, Result};
use orchestrator::{
    assert_llm_egress_allowed, execute_command, health::probe_services, set_channel_enabled,
    set_primary_llm, set_security_profile, ChannelCatalog, Command as BridgeCommand,
    OrchestratorConfig, OrchestratorFacade, Response,
};
use orchestrator::gateway::resolve_channel_config;
use reqwest::Client;
use tokio::time::timeout;

use crate::output::print_response;

const DAEMON_TASK_NAME: &str = "OrchestrateurDaemon";
const DAEMON_TOKEN_ENV: &str = "ORCHESTRATEUR_DAEMON_TOKEN";

/// Options d'onboard interactif ou via flags.
#[derive(Debug, Clone)]
pub struct OnboardOptions {
    /// Profil sécurité TOML (`local_only`, `ai_assisted`, …).
    pub profile: Option<String>,
    /// Provider LLM primaire.
    pub llm: Option<String>,
    /// Raccourci profil local souverain.
    pub local_only: bool,
    /// Installe la tâche planifiée Windows après onboard.
    pub install_daemon: bool,
}

/// Options de configuration harness.
#[derive(Debug, Clone, Default)]
pub struct ConfigureOptions {
    pub profile: Option<String>,
    pub llm: Option<String>,
    pub local_only: bool,
}

/// Sonde HTTP daemon/gateway.
#[derive(Debug, serde::Deserialize)]
struct ProbeHealth {
    status: String,
    version: String,
    port: u16,
}

/// Diagnostic harness enrichi (Cortex + services + tokens + egress).
pub async fn cmd_doctor(facade: &OrchestratorFacade, workspace: &Path) -> Result<()> {
    let config = OrchestratorConfig::load_workspace(workspace)
        .map_err(|e| anyhow::anyhow!("config: {e}"))?;

    println!("=== Orchestrateur harness intégral ===");
    let mut issues = 0usize;

    macro_rules! check {
        ($label:expr, $expr:expr) => {{
            match $expr {
                Ok(()) => println!("  ✓ {label}", label = $label),
                Err(err) => {
                    println!("  ✗ {label}: {err}", label = $label);
                    issues += 1;
                }
            }
        }};
    }

    check!(
        "health bridge",
        run_bridge(facade, BridgeCommand::HealthCheck).await
    );
    check!("graphe", run_bridge(facade, BridgeCommand::Graph).await);
    check!(
        "watcher",
        run_bridge(facade, BridgeCommand::WatcherStatus).await
    );
    check!(
        "drafts",
        run_bridge(facade, BridgeCommand::ListDrafts).await
    );

    let probe = probe_services(facade.deps()).await;
    if probe.llm_available {
        println!("  ✓ LLM probe");
    } else {
        println!("  ⚠ LLM indisponible (optionnel si mode headless)");
    }
    if probe.embedding_available {
        println!("  ✓ embedding probe");
    } else {
        println!("  ⚠ embedding indisponible");
        issues += 1;
    }

    if let Some(profile) = &config.security.profile {
        println!("  profil sécurité : {profile:?}");
    }
    if config.security.block_cloud_llm {
        println!("  egress : block_cloud_llm=true");
        if let Err(err) = assert_llm_egress_allowed(&config) {
            println!("  ✗ egress LLM : {}", err.message);
            issues += 1;
        } else {
            println!("  ✓ egress LLM local");
        }
    }

    check_env_token(&config.daemon.token_env, "daemon token");
    check_env_token(&config.gateway.token_env, "gateway token");

    let client = http_client();
    probe_service(
        &client,
        "daemon",
        &format!("http://{}:{}/health", config.daemon.bind, config.daemon.port),
        config.daemon.enabled,
    )
    .await;
    probe_service(
        &client,
        "gateway",
        &format!("http://{}:{}/health", config.gateway.bind, config.gateway.port),
        config.gateway.enabled,
    )
    .await;

    let enabled_channels = list_enabled_channels(&config);
    if enabled_channels.is_empty() {
        println!("  canaux activés : (aucun)");
    } else {
        println!("  canaux activés : {}", enabled_channels.join(", "));
    }

    if issues == 0 {
        println!("doctor: ok");
        Ok(())
    } else {
        anyhow::bail!("doctor: {issues} problème(s) détecté(s)")
    }
}

async fn run_bridge(facade: &OrchestratorFacade, command: BridgeCommand) -> Result<()> {
    let response = execute_command(facade, command).await;
    print_response(response)
}

fn check_env_token(env_name: &str, label: &str) {
    if env_name.is_empty() {
        println!("  ⚠ {label} : variable non configurée");
        return;
    }
    match std::env::var(env_name) {
        Ok(_) => println!("  ✓ {label} ({env_name})"),
        Err(_) => println!("  ✗ {label} absent — définir {env_name}"),
    }
}

fn http_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap_or_else(|_| Client::new())
}

async fn probe_service(client: &Client, name: &str, url: &str, enabled: bool) {
    if !enabled {
        println!("  ⚠ {name} désactivé dans orchestrator.toml");
        return;
    }
    match timeout(Duration::from_secs(3), client.get(url).send()).await {
        Ok(Ok(resp)) if resp.status().is_success() => {
            if let Ok(body) = resp.json::<ProbeHealth>().await {
                println!(
                    "  ✓ {name} actif — v{} port {}",
                    body.version, body.port
                );
            } else {
                println!("  ✓ {name} actif ({url})");
            }
        }
        Ok(Ok(resp)) => {
            println!("  ✗ {name} HTTP {}", resp.status());
        }
        Ok(Err(err)) => {
            println!("  ⚠ {name} injoignable ({url}) — {err}");
        }
        Err(_) => {
            println!("  ⚠ {name} timeout ({url})");
        }
    }
}

fn list_enabled_channels(config: &OrchestratorConfig) -> Vec<String> {
    let catalog = ChannelCatalog::new();
    catalog
        .descriptors()
        .iter()
        .filter(|d| resolve_channel_config(&config.gateway, d.id).enabled)
        .map(|d| d.id.to_string())
        .collect()
}

/// Onboard workspace — structure, config, token daemon, tâche optionnelle.
pub fn cmd_onboard(workspace: &Path, options: &OnboardOptions) -> Result<()> {
    ensure_workspace_tree(workspace)?;

    let settings = workspace.join("config").join("orchestrator.toml");
    if !settings.exists() {
        let example = find_example_config();
        if let Some(src) = example {
            std::fs::copy(&src, &settings)
                .with_context(|| format!("copie config depuis {}", src.display()))?;
            println!("Config créée depuis {}", src.display());
        } else {
            write_minimal_config(&settings)?;
            println!("Config minimale créée : {}", settings.display());
        }
    }

    let profile = resolve_profile(options);
    set_security_profile(&settings, &profile)
        .map_err(|e| anyhow::anyhow!("profil: {e}"))?;
    println!("Profil sécurité : {profile}");

    if let Some(llm) = options.llm.as_deref().or_else(|| {
        if options.local_only || profile == "local_only" {
            Some("ollama")
        } else {
            None
        }
    }) {
        set_primary_llm(&settings, llm).map_err(|e| anyhow::anyhow!("llm: {e}"))?;
        println!("Provider LLM : {llm}");
    }

    ensure_daemon_token_user()?;

    if options.install_daemon {
        daemon_install(workspace)?;
    }

    println!("Onboard terminé — workspace : {}", workspace.display());
    println!("Prochaine étape : orch doctor --workspace {}", workspace.display());
    Ok(())
}

/// Met à jour des champs harness dans orchestrator.toml.
pub fn cmd_configure(workspace: &Path, options: &ConfigureOptions) -> Result<()> {
    let settings = workspace.join("config").join("orchestrator.toml");
    if !settings.exists() {
        anyhow::bail!(
            "config absente — lancez d'abord : orch onboard --workspace {}",
            workspace.display()
        );
    }

    if options.local_only || options.profile.is_some() {
        let profile = if options.local_only {
            "local_only".to_string()
        } else {
            options.profile.clone().unwrap_or_else(|| "ai_assisted".into())
        };
        set_security_profile(&settings, &profile)
            .map_err(|e| anyhow::anyhow!("profil: {e}"))?;
        println!("Profil : {profile}");
    }

    let llm = options
        .llm
        .as_deref()
        .or(if options.local_only { Some("ollama") } else { None });
    if let Some(provider) = llm {
        set_primary_llm(&settings, provider).map_err(|e| anyhow::anyhow!("llm: {e}"))?;
        println!("Provider LLM : {provider}");
    }

    println!("Configuration mise à jour.");
    Ok(())
}

/// Installe la tâche planifiée Windows pour le daemon.
pub fn daemon_install(workspace: &Path) -> Result<()> {
    ensure_daemon_token_user()?;
    let exe = std::env::current_exe().context("binaire CLI introuvable")?;
    let ws = workspace.to_string_lossy();
    let args = format!("daemon run --workspace \"{ws}\"");

    #[cfg(windows)]
    {
        let tr = format!("\"{}\" {}", exe.display(), args);
        let status = OsCommand::new("schtasks")
            .args([
                "/Create",
                "/F",
                "/TN",
                DAEMON_TASK_NAME,
                "/TR",
                &tr,
                "/SC",
                "ONLOGON",
                "/RL",
                "LIMITED",
            ])
            .status()
            .context("schtasks /Create")?;
        if status.success() {
            println!("Tâche planifiée installée : {DAEMON_TASK_NAME}");
            Ok(())
        } else {
            anyhow::bail!("échec schtasks (code {status})")
        }
    }

    #[cfg(not(windows))]
    {
        println!("daemon install : plateforme non Windows");
        println!("Lancez manuellement : {} daemon run --workspace {ws}", exe.display());
        Ok(())
    }
}

/// Statut daemon — tâche planifiée + sonde HTTP.
pub async fn daemon_status(workspace: &Path) -> Result<()> {
    let config = OrchestratorConfig::load_workspace(workspace)
        .map_err(|e| anyhow::anyhow!("config: {e}"))?;

    #[cfg(windows)]
    {
        let output = OsCommand::new("schtasks")
            .args(["/Query", "/TN", DAEMON_TASK_NAME, "/FO", "LIST"])
            .output();
        match output {
            Ok(out) if out.status.success() => {
                println!("tâche planifiée : {DAEMON_TASK_NAME} (installée)");
            }
            _ => println!("tâche planifiée : absente (orch daemon install)"),
        }
    }

    let url = format!("http://{}:{}/health", config.daemon.bind, config.daemon.port);
    let client = http_client();
    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let body: ProbeHealth = resp.json().await.unwrap_or(ProbeHealth {
                status: "ok".into(),
                version: "?".into(),
                port: config.daemon.port,
            });
            println!(
                "daemon : actif — {} v{} port {}",
                body.status, body.version, body.port
            );
        }
        Ok(resp) => println!("daemon : HTTP {}", resp.status()),
        Err(err) => println!("daemon : arrêté ou injoignable ({url}) — {err}"),
    }
    Ok(())
}

/// Désinstallation complète Windows (script détaché — le binaire courant se libère).
pub fn cmd_uninstall() -> Result<()> {
    use crate::windows_ops::{powershell_uninstall_body, spawn_detached_after_exit};

    println!("Lancement de la désinstallation complète…");
    daemon_stop()?;
    let task = powershell_uninstall_body();
    spawn_detached_after_exit(&task)?;
    println!();
    println!("Désinstallation lancée en arrière-plan.");
    println!("PATH, binaires et tâche planifiée seront retirés automatiquement.");
    println!("Options avancées : uninstall.ps1 -PurgeData -AllUsers");
    std::process::exit(0);
}

#[cfg(windows)]
fn stop_orchestrateur_processes_except_current() -> bool {
    let current = std::process::id();
    let script = format!(
        "$names = 'orch.exe','orchestrateur.exe','orchestre.exe'; \
         $stopped = $false; \
         foreach ($name in $names) {{ \
           Get-CimInstance Win32_Process -Filter \"name='$name'\" | \
           Where-Object {{ $_.ProcessId -ne {current} }} | \
           ForEach-Object {{ Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue; $stopped = $true }} \
         }}; if ($stopped) {{ exit 0 }} else {{ exit 1 }}"
    );
    OsCommand::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Arrête le daemon (tâche planifiée + processus orch daemon run).
pub fn daemon_stop() -> Result<()> {
    #[cfg(windows)]
    {
        let _ = OsCommand::new("schtasks")
            .args(["/End", "/TN", DAEMON_TASK_NAME])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let stopped = stop_orchestrateur_processes_except_current();
        if stopped {
            println!("Daemon arrêté.");
        } else {
            println!("Aucun processus daemon trouvé (ou arrêt manuel requis).");
        }
    }

    #[cfg(not(windows))]
    {
        println!("daemon stop : plateforme non Windows");
    }
    Ok(())
}

/// Statut gateway via sonde HTTP /health.
pub async fn gateway_status(workspace: &Path) -> Result<()> {
    let config = OrchestratorConfig::load_workspace(workspace)
        .map_err(|e| anyhow::anyhow!("config: {e}"))?;
    if !config.gateway.enabled {
        println!("gateway : désactivé dans orchestrator.toml");
        return Ok(());
    }
    let url = format!("http://{}:{}/health", config.gateway.bind, config.gateway.port);
    let client = http_client();
    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let body: ProbeHealth = resp.json().await.unwrap_or(ProbeHealth {
                status: "ok".into(),
                version: "?".into(),
                port: config.gateway.port,
            });
            println!(
                "gateway : actif — {} v{} port {}",
                body.status, body.version, body.port
            );
        }
        Ok(resp) => println!("gateway : HTTP {}", resp.status()),
        Err(err) => println!("gateway : arrêté ou injoignable ({url}) — {err}"),
    }
    Ok(())
}

/// Active un canal gateway dans orchestrator.toml.
pub fn channels_enable(workspace: &Path, channel_id: &str) -> Result<()> {
    let settings = config_path(workspace)?;
    set_channel_enabled(&settings, channel_id, true)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("Canal {channel_id} activé.");
    Ok(())
}

/// Désactive un canal gateway.
pub fn channels_disable(workspace: &Path, channel_id: &str) -> Result<()> {
    let settings = config_path(workspace)?;
    set_channel_enabled(&settings, channel_id, false)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("Canal {channel_id} désactivé.");
    Ok(())
}

/// Statut des canaux (enabled + token env).
pub fn channels_status(workspace: &Path) -> Result<()> {
    let config = OrchestratorConfig::load_workspace(workspace)
        .map_err(|e| anyhow::anyhow!("config: {e}"))?;
    let catalog = ChannelCatalog::new();
    println!("# Canaux ({})", catalog.count());
    for descriptor in catalog.descriptors() {
        let cfg = resolve_channel_config(&config.gateway, descriptor.id);
        let state = if cfg.enabled { "on" } else { "off" };
        let token = if cfg.token_env.is_empty() {
            "n/a".to_string()
        } else if std::env::var(&cfg.token_env).is_ok() {
            format!("{}=set", cfg.token_env)
        } else {
            format!("{}=missing", cfg.token_env)
        };
        let kind = if descriptor.dedicated { "live" } else { "stub" };
        println!(
            "{:<14} {:<4} {:<6} env={token} [{kind}]",
            descriptor.id, state, descriptor.display_name
        );
    }
    Ok(())
}

/// Sonde un provider LLM ou embedding.
pub async fn providers_test(facade: &OrchestratorFacade, kind: Option<&str>) -> Result<()> {
    let probe = probe_services(facade.deps()).await;
    match kind {
        Some("llm") => {
            println!(
                "LLM ({}): {}",
                facade.deps().config.providers.primary_llm,
                if probe.llm_available { "ok" } else { "échec" }
            );
            if !probe.llm_available {
                anyhow::bail!("probe LLM échouée");
            }
        }
        Some("embedding") => {
            println!(
                "Embedding ({}): {}",
                facade.deps().config.providers.primary_embedding,
                if probe.embedding_available {
                    "ok"
                } else {
                    "échec"
                }
            );
            if !probe.embedding_available {
                anyhow::bail!("probe embedding échouée");
            }
        }
        Some(other) => anyhow::bail!("kind inconnu: {other} (llm ou embedding)"),
        None => {
            println!(
                "LLM ({}) : {}",
                facade.deps().config.providers.primary_llm,
                if probe.llm_available { "ok" } else { "échec" }
            );
            println!(
                "Embedding ({}) : {}",
                facade.deps().config.providers.primary_embedding,
                if probe.embedding_available {
                    "ok"
                } else {
                    "échec"
                }
            );
            if !probe.llm_available && !probe.embedding_available {
                anyhow::bail!("aucun provider joignable");
            }
        }
    }
    Ok(())
}

/// Définit le provider LLM primaire.
pub fn providers_set(workspace: &Path, provider: &str) -> Result<()> {
    let settings = config_path(workspace)?;
    set_primary_llm(&settings, provider).map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("Provider LLM primaire : {provider}");
    Ok(())
}

/// Smoke harness (health, graph, watcher, drafts, memories).
pub async fn cmd_harness_smoke(facade: &OrchestratorFacade) -> Result<()> {
    println!("harness smoke…");
    let steps: &[(&str, BridgeCommand)] = &[
        ("health", BridgeCommand::HealthCheck),
        ("graph", BridgeCommand::Graph),
        ("watcher", BridgeCommand::WatcherStatus),
        ("drafts", BridgeCommand::ListDrafts),
        (
            "memories",
            BridgeCommand::List {
                filter: None,
                offset: 0,
                limit: 3,
            },
        ),
    ];
    for (name, cmd) in steps {
        let response = execute_command(facade, cmd.clone()).await;
        if matches!(response, Response::Error(_)) {
            print_response(response)?;
            anyhow::bail!("harness smoke échoué à l'étape: {name}");
        }
        println!("  ✓ {name}");
    }
    println!("harness smoke: ok");
    Ok(())
}

/// Superviseur harness — démarre daemon + gateway si absents, attend Ctrl+C.
pub async fn cmd_harness_run(workspace: &Path) -> Result<()> {
    let config = OrchestratorConfig::load_workspace(workspace)
        .map_err(|e| anyhow::anyhow!("config: {e}"))?;
    ensure_daemon_token_user()?;

    let exe = std::env::current_exe().context("binaire CLI")?;
    let ws = workspace.to_string_lossy().to_string();
    let mut children = Vec::new();

    let daemon_url = format!("http://{}:{}/health", config.daemon.bind, config.daemon.port);
    if config.daemon.enabled && !service_alive(&daemon_url).await {
        let child = OsCommand::new(&exe)
            .args(["daemon", "run", "--workspace", &ws])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("spawn daemon")?;
        children.push(child);
        tokio::time::sleep(Duration::from_secs(2)).await;
        println!("daemon démarré ({daemon_url})");
    } else if config.daemon.enabled {
        println!("daemon déjà actif");
    }

    let gateway_url = format!("http://{}:{}/health", config.gateway.bind, config.gateway.port);
    if config.gateway.enabled && !service_alive(&gateway_url).await {
        let child = OsCommand::new(&exe)
            .args(["gateway", "run", "--workspace", &ws])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("spawn gateway")?;
        children.push(child);
        tokio::time::sleep(Duration::from_secs(2)).await;
        println!("gateway démarré ({gateway_url})");
    } else if config.gateway.enabled {
        println!("gateway déjà actif");
    }

    println!("Harness run actif — Ctrl+C pour arrêter les processus enfants.");
    tokio::signal::ctrl_c()
        .await
        .context("ctrl+c")?;

    for mut child in children {
        let _ = child.kill();
    }
    println!("Harness run arrêté.");
    Ok(())
}

async fn service_alive(url: &str) -> bool {
    http_client()
        .get(url)
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

fn config_path(workspace: &Path) -> Result<PathBuf> {
    let path = workspace.join("config").join("orchestrator.toml");
    if path.exists() {
        Ok(path)
    } else {
        anyhow::bail!("config absente : {}", path.display())
    }
}

fn ensure_workspace_tree(workspace: &Path) -> Result<()> {
    for dir in [
        workspace,
        &workspace.join("memories"),
        &workspace.join("logs"),
        &workspace.join("config"),
        &workspace.join(".orchestrateur").join("sessions"),
        &workspace.join(".orchestrateur").join("drafts"),
    ] {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("mkdir {}", dir.display()))?;
    }
    Ok(())
}

fn find_example_config() -> Option<PathBuf> {
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

fn write_minimal_config(path: &Path) -> Result<()> {
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
    std::fs::write(path, body).with_context(|| format!("écriture {}", path.display()))?;
    Ok(())
}

fn resolve_profile(options: &OnboardOptions) -> String {
    if options.local_only {
        return "local_only".into();
    }
    if let Some(p) = &options.profile {
        return p.clone();
    }
    "ai_assisted".into()
}

fn ensure_daemon_token_user() -> Result<()> {
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
            .context("lecture token utilisateur")?;
        let existing = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !existing.is_empty() {
            std::env::set_var(DAEMON_TOKEN_ENV, &existing);
            return Ok(());
        }
        let token = uuid::Uuid::new_v4().as_simple().to_string();
        OsCommand::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "[Environment]::SetEnvironmentVariable('{DAEMON_TOKEN_ENV}', '{token}', 'User')"
                ),
            ])
            .status()
            .context("écriture token utilisateur")?;
        std::env::set_var(DAEMON_TOKEN_ENV, &token);
        println!("Token daemon généré ({DAEMON_TOKEN_ENV})");
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let token = uuid::Uuid::new_v4().as_simple().to_string();
        std::env::set_var(DAEMON_TOKEN_ENV, &token);
        println!("Token daemon (session) : {DAEMON_TOKEN_ENV}={token}");
        Ok(())
    }
}

