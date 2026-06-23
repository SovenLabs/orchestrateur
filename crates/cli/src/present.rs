//! Affichage CLI — formate les résultats métier `orchestrator::harness`.

use std::path::Path;

use anyhow::Result;
use orchestrator::{
    disable_channel, enable_channel, harness::config_path, install_scheduled_task,
    list_channel_status, plan_supervisor, probe_daemon_status, probe_gateway_status,
    probe_harness_services, probe_health, probe_providers, run_configure, run_doctor,
    run_onboard, run_smoke, scheduled_task_installed, set_primary_llm,
    spawn_child, stop_daemon, validate_probe, wait_for_spawn, ChannelStatusRow, CheckStatus,
    ConfigureOptions, HarnessError, HarnessSmokeOptions, OnboardOptions, OrchestratorConfig,
    OrchestratorFacade, ServiceHealth, ServiceProbeState, ServiceStatusDetail,
};

pub async fn harness_service_badges(workspace: &Path) -> (ServiceProbeState, ServiceProbeState) {
    orchestrator::service_badges(workspace).await
}

pub async fn probe_daemon_health(url: &str) -> ServiceProbeState {
    let client = orchestrator::harness::probe_client();
    match probe_health(&client, url).await {
        ServiceHealth::Alive => ServiceProbeState::Alive,
        ServiceHealth::Down => ServiceProbeState::Down,
    }
}

pub async fn probe_gateway_health(url: &str) -> ServiceProbeState {
    probe_daemon_health(url).await
}

pub fn onboard(workspace: &Path, options: &OnboardOptions) -> Result<()> {
    let result = run_onboard(workspace, options).map_err(anyhow::Error::from)?;
    if result.daemon_token_generated {
        println!("Token daemon généré (ORCHESTRATEUR_DAEMON_TOKEN)");
    }
    if let Some(llm) = &result.llm {
        println!("Provider LLM : {llm}");
    }
    println!("Profil sécurité : {}", result.profile);
    if result.daemon_task_installed {
        println!("Tâche planifiée daemon installée");
    }
    println!("Onboard terminé — workspace : {}", result.workspace_display);
    println!("Prochaine étape : orch doctor --workspace {}", result.workspace_display);
    Ok(())
}

pub fn configure(workspace: &Path, options: &ConfigureOptions) -> Result<()> {
    run_configure(workspace, options).map_err(anyhow::Error::from)?;
    println!("Configuration mise à jour.");
    Ok(())
}

pub async fn doctor(facade: &OrchestratorFacade, workspace: &Path) -> Result<()> {
    let report = run_doctor(facade, workspace).await.map_err(anyhow::Error::from)?;
    print_doctor(&report);
    if !report.is_ok() {
        anyhow::bail!("doctor: {} problème(s) détecté(s)", report.issue_count);
    }
    println!("doctor: ok");
    Ok(())
}

fn print_doctor(report: &orchestrator::DoctorReport) {
    println!("=== Orchestrateur harness intégral ===");
    for check in &report.checks {
        let icon = match check.status {
            CheckStatus::Ok => "✓",
            CheckStatus::Warn => "⚠",
            CheckStatus::Fail => "✗",
        };
        if let Some(detail) = &check.detail {
            println!("  {icon} {} — {detail}", check.label);
        } else {
            println!("  {icon} {}", check.label);
        }
    }
    if report.enabled_channels.is_empty() {
        println!("  canaux activés : (aucun)");
    } else {
        println!("  canaux activés : {}", report.enabled_channels.join(", "));
    }
}

pub fn daemon_install(workspace: &Path) -> Result<()> {
    let result = install_scheduled_task(workspace).map_err(anyhow::Error::from)?;
    if result.installed {
        println!("Tâche planifiée installée : {}", result.task_name);
    } else {
        println!("daemon install : plateforme non Windows — lancez `orch daemon start`");
    }
    Ok(())
}

pub async fn daemon_status(workspace: &Path) -> Result<()> {
    if scheduled_task_installed() {
        println!("tâche planifiée : OrchestrateurDaemon (installée)");
    } else {
        println!("tâche planifiée : absente (orch daemon install)");
    }
    let detail = probe_daemon_status(workspace).await.map_err(anyhow::Error::from)?;
    print_service_status(&detail);
    Ok(())
}

pub fn daemon_stop() -> Result<()> {
    let result = stop_daemon().map_err(anyhow::Error::from)?;
    if result.stopped {
        println!("Daemon arrêté.");
    } else {
        println!("Aucun processus daemon trouvé.");
    }
    Ok(())
}

pub async fn gateway_status(workspace: &Path) -> Result<()> {
    let detail = probe_gateway_status(workspace).await.map_err(anyhow::Error::from)?;
    print_service_status(&detail);
    Ok(())
}

fn print_service_status(detail: &ServiceStatusDetail) {
    match detail.state.as_str() {
        "active" => {
            let version = detail.version.as_deref().unwrap_or("?");
            let port = detail.port.map(|p| p.to_string()).unwrap_or_else(|| "?".into());
            println!(
                "{} : actif — {} v{} port {}",
                detail.name,
                detail.detail.as_deref().unwrap_or("ok"),
                version,
                port
            );
        }
        "disabled" => println!("{} : désactivé dans orchestrator.toml", detail.name),
        "http_error" => println!(
            "{} : HTTP {}",
            detail.name,
            detail.detail.as_deref().unwrap_or("?")
        ),
        _ => println!(
            "{} : arrêté ou injoignable ({}) — {}",
            detail.name,
            detail.url,
            detail.detail.as_deref().unwrap_or("?")
        ),
    }
}

pub fn channels_enable(workspace: &Path, channel_id: &str) -> Result<()> {
    enable_channel(workspace, channel_id).map_err(anyhow::Error::from)?;
    println!("Canal {channel_id} activé.");
    Ok(())
}

pub fn channels_disable(workspace: &Path, channel_id: &str) -> Result<()> {
    disable_channel(workspace, channel_id).map_err(anyhow::Error::from)?;
    println!("Canal {channel_id} désactivé.");
    Ok(())
}

pub fn channels_status(workspace: &Path) -> Result<()> {
    let rows = list_channel_status(workspace).map_err(anyhow::Error::from)?;
    println!("# Canaux ({})", rows.len());
    for row in &rows {
        print_channel_row(row);
    }
    Ok(())
}

fn print_channel_row(row: &ChannelStatusRow) {
    let state = if row.enabled { "on" } else { "off" };
    println!(
        "{:<14} {:<4} {:<6} env={} [{}]",
        row.id, state, row.display_name, row.token_state, row.kind
    );
}

pub async fn providers_test(facade: &OrchestratorFacade, kind: Option<&str>) -> Result<()> {
    let result = probe_providers(facade).await;
    println!(
        "LLM ({}) : {}",
        result.llm_id,
        if result.llm_ok { "ok" } else { "échec" }
    );
    println!(
        "Embedding ({}) : {}",
        result.embedding_id,
        if result.embedding_ok { "ok" } else { "échec" }
    );
    validate_probe(&result, kind).map_err(anyhow::Error::from)?;
    Ok(())
}

pub fn providers_set(workspace: &Path, provider: &str) -> Result<()> {
    let settings = config_path(workspace);
    set_primary_llm(&settings, provider).map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("Provider LLM primaire : {provider}");
    Ok(())
}

pub async fn harness_smoke(
    facade: &OrchestratorFacade,
    workspace: &Path,
    opts: &HarnessSmokeOptions,
) -> Result<()> {
    println!("harness smoke…");
    if !opts.skip_gateway {
        let config = OrchestratorConfig::load_workspace(workspace)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        if config.gateway.enabled {
            let probe = probe_harness_services(&config, &orchestrator::harness::probe_client()).await;
            if probe.gateway != "alive" && probe.gateway != "skipped" {
                anyhow::bail!(
                    "gateway injoignable ({}) — relancez harness run ou --skip-gateway",
                    probe.gateway_url
                );
            }
            println!("  ✓ gateway probe");
        }
    }
    let result = run_smoke(facade, workspace, opts)
        .await
        .map_err(|e| match e {
            HarnessError::SmokeFailed { step } => {
                anyhow::anyhow!("harness smoke échoué à l'étape: {step}")
            }
            HarnessError::GatewayDown { url } => {
                anyhow::anyhow!("gateway injoignable ({url})")
            }
            other => anyhow::Error::from(other),
        })?;
    for step in &result.steps {
        println!("  ✓ {step}");
    }
    println!("harness smoke: ok");
    Ok(())
}

pub async fn harness_run(workspace: &Path) -> Result<()> {
    let plan = plan_supervisor(workspace)
        .await
        .map_err(anyhow::Error::from)?;
    let ws = workspace.to_string_lossy().to_string();
    let mut children = Vec::new();

    if plan.spawn_daemon {
        let child = spawn_child(&["daemon", "start", "--workspace", &ws])
            .map_err(anyhow::Error::from)?;
        children.push(child);
        wait_for_spawn().await;
        println!("daemon démarré ({})", plan.daemon_url);
    } else if !plan.daemon_url.is_empty() {
        println!("daemon déjà actif");
    }

    if plan.spawn_gateway {
        let child = spawn_child(&["gateway", "run", "--workspace", &ws])
            .map_err(anyhow::Error::from)?;
        children.push(child);
        wait_for_spawn().await;
        println!("gateway démarré ({})", plan.gateway_url);
    } else if !plan.gateway_url.is_empty() {
        println!("gateway déjà actif");
    }

    println!("Harness run actif — Ctrl+C pour arrêter les processus enfants.");
    tokio::signal::ctrl_c()
        .await
        .map_err(|e| anyhow::anyhow!("ctrl+c: {e}"))?;

    for mut child in children {
        let _ = child.kill();
    }
    println!("Harness run arrêté.");
    Ok(())
}

pub fn set_user_env_var(name: &str, value: &str) -> Result<()> {
    orchestrator::set_user_env_var(name, value).map_err(anyhow::Error::from)
}

pub fn uninstall() -> Result<()> {
    use crate::windows_ops::{powershell_uninstall_body, spawn_detached_after_exit};

    println!("Lancement de la désinstallation complète…");
    daemon_stop()?;
    let task = powershell_uninstall_body();
    spawn_detached_after_exit(&task)?;
    println!();
    println!("Désinstallation lancée en arrière-plan.");
    println!("PATH, binaires et tâche planifiée seront retirés automatiquement.");
    std::process::exit(0);
}