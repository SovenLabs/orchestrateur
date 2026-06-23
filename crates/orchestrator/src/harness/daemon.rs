//! Opérations daemon (tâche planifiée Windows, arrêt processus).

use std::path::Path;
use std::process::{Command as OsCommand, Stdio};

use crate::config::OrchestratorConfig;
use crate::harness::client::probe_client;
use crate::harness::env::ensure_daemon_token;
use crate::harness::error::HarnessError;
use crate::harness::types::{DaemonInstallResult, DaemonStopResult, HealthBody, ServiceStatusDetail};

const DAEMON_TASK_NAME: &str = "OrchestrateurDaemon";

/// Installe la tâche planifiée Windows pour le daemon.
pub fn install_scheduled_task(workspace: &Path) -> Result<DaemonInstallResult, HarnessError> {
    ensure_daemon_token()?;
    let exe = std::env::current_exe().map_err(|e| HarnessError::Platform(e.to_string()))?;
    let ws = workspace.to_string_lossy();
    let args = format!("daemon start --workspace \"{ws}\"");

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
            .map_err(|e| HarnessError::Platform(e.to_string()))?;
        if status.success() {
            Ok(DaemonInstallResult {
                installed: true,
                task_name: DAEMON_TASK_NAME.into(),
            })
        } else {
            Err(HarnessError::Platform(format!("schtasks échec ({status})")))
        }
    }

    #[cfg(not(windows))]
    {
        let _ = (exe, args);
        Ok(DaemonInstallResult {
            installed: false,
            task_name: DAEMON_TASK_NAME.into(),
        })
    }
}

/// Arrête le daemon (tâche + processus orch).
pub fn stop_daemon() -> Result<DaemonStopResult, HarnessError> {
    #[cfg(windows)]
    {
        let _ = OsCommand::new("schtasks")
            .args(["/End", "/TN", DAEMON_TASK_NAME])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let stopped = stop_orchestrateur_processes_except_current();
        Ok(DaemonStopResult { stopped })
    }

    #[cfg(not(windows))]
    {
        Ok(DaemonStopResult { stopped: false })
    }
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
        .map_or(false, |s| s.success())
}

/// Tâche planifiée installée (Windows).
#[must_use]
pub fn scheduled_task_installed() -> bool {
    #[cfg(windows)]
    {
        OsCommand::new("schtasks")
            .args(["/Query", "/TN", DAEMON_TASK_NAME, "/FO", "LIST"])
            .output()
            .map_or(false, |out| out.status.success())
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Sonde HTTP statut daemon.
pub async fn probe_daemon_status(workspace: &Path) -> Result<ServiceStatusDetail, HarnessError> {
    let config = OrchestratorConfig::load_workspace(workspace)?;
    let url = format!("http://{}:{}/health", config.daemon.bind, config.daemon.port);
    Ok(probe_service_url("daemon", &url, config.daemon.port).await)
}

/// Sonde HTTP statut gateway.
pub async fn probe_gateway_status(workspace: &Path) -> Result<ServiceStatusDetail, HarnessError> {
    let config = OrchestratorConfig::load_workspace(workspace)?;
    if !config.gateway.enabled {
        return Ok(ServiceStatusDetail {
            name: "gateway".into(),
            state: "disabled".into(),
            version: None,
            port: Some(config.gateway.port),
            url: format!("http://{}:{}/health", config.gateway.bind, config.gateway.port),
            detail: None,
        });
    }
    let url = format!("http://{}:{}/health", config.gateway.bind, config.gateway.port);
    Ok(probe_service_url("gateway", &url, config.gateway.port).await)
}

pub(crate) async fn probe_service_url(name: &str, url: &str, default_port: u16) -> ServiceStatusDetail {
    let client = probe_client();
    match client.get(url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let parsed = resp.json::<HealthBody>().await.ok();
            ServiceStatusDetail {
                name: name.into(),
                state: "active".into(),
                version: parsed.as_ref().map(|b| b.version.clone()),
                port: parsed.as_ref().map(|b| b.port).or(Some(default_port)),
                url: url.into(),
                detail: parsed.map(|b| b.status),
            }
        }
        Ok(resp) => ServiceStatusDetail {
            name: name.into(),
            state: "http_error".into(),
            version: None,
            port: Some(default_port),
            url: url.into(),
            detail: Some(resp.status().to_string()),
        },
        Err(err) => ServiceStatusDetail {
            name: name.into(),
            state: "down".into(),
            version: None,
            port: Some(default_port),
            url: url.into(),
            detail: Some(err.to_string()),
        },
    }
}

/// Badges daemon + gateway pour menus.
pub async fn service_badges(workspace: &Path) -> (crate::harness::types::ServiceProbeState, crate::harness::types::ServiceProbeState) {
    use crate::harness::probe::probe_harness_services;
    use crate::harness::types::ServiceProbeState;

    let Ok(config) = OrchestratorConfig::load_workspace(workspace) else {
        return (ServiceProbeState::Unknown, ServiceProbeState::Unknown);
    };
    let probe = probe_harness_services(&config, &probe_client()).await;
    (
        ServiceProbeState::from_probe_status(&probe.daemon),
        ServiceProbeState::from_probe_status(&probe.gateway),
    )
}