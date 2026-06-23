//! Scheduler cron local (`.orchestrateur/cron/jobs.json`).

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::facade::OrchestratorFacade;
use crate::persistent::PersistentAgentError;

/// Job cron persisté.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobRecord {
    /// Identifiant.
    pub id: String,
    /// Nom affiché.
    pub name: String,
    /// Prompt exécuté.
    pub prompt: String,
    /// Schedule (`every:300s`, `*/5m`, `@daily`).
    pub schedule: String,
    /// Actif.
    pub enabled: bool,
    /// Création ISO-8601.
    pub created_at: String,
    /// Dernière exécution.
    #[serde(default)]
    pub last_run: Option<String>,
    /// Prochaine exécution planifiée.
    #[serde(default)]
    pub next_run: Option<String>,
    /// Agent cible.
    #[serde(default)]
    pub agent_id: Option<String>,
}

/// Rapport d'exécution cron.
#[derive(Debug, Clone, Default)]
pub struct CronTickReport {
    /// Jobs exécutés.
    pub ran: usize,
    /// Jobs ignorés (pas encore dus).
    pub skipped: usize,
}

pub fn cron_jobs_path(workspace: &Path) -> PathBuf {
    workspace.join(".orchestrateur").join("cron").join("jobs.json")
}

/// Exécute les jobs cron dus.
pub async fn run_due_cron_jobs(
    facade: &OrchestratorFacade,
    workspace: &Path,
    default_agent_id: &str,
) -> Result<CronTickReport, PersistentAgentError> {
    let path = cron_jobs_path(workspace);
    if !path.is_file() {
        return Ok(CronTickReport::default());
    }

    let raw = tokio::fs::read_to_string(&path).await.map_err(|e| {
        PersistentAgentError::Io(e.to_string())
    })?;
    let mut jobs: Vec<CronJobRecord> =
        serde_json::from_str(&raw).unwrap_or_default();
    let now = Utc::now();
    let mut report = CronTickReport::default();

    for job in &mut jobs {
        if !job.enabled {
            continue;
        }
        if !is_due(job, now) {
            report.skipped += 1;
            continue;
        }
        let agent_id = job.agent_id.as_deref().unwrap_or(default_agent_id);
        let prompt = format!("[Cron {}] {}", job.name, job.prompt);
        match facade.agent_turn_for(agent_id, &prompt).await {
            Ok(_) => {
                job.last_run = Some(now.to_rfc3339());
                job.next_run = Some(next_run_iso(&job.schedule, now));
                report.ran += 1;
            }
            Err(err) => {
                tracing::warn!(job = %job.id, %err, "cron job échoué");
                job.last_run = Some(now.to_rfc3339());
                report.skipped += 1;
            }
        }
    }

    let updated = serde_json::to_string_pretty(&jobs)
        .map_err(|e| PersistentAgentError::Config(e.to_string()))?;
    tokio::fs::write(&path, updated)
        .await
        .map_err(|e| PersistentAgentError::Io(e.to_string()))?;
    Ok(report)
}

fn is_due(job: &CronJobRecord, now: DateTime<Utc>) -> bool {
    if let Some(next) = &job.next_run {
        if let Ok(dt) = DateTime::parse_from_rfc3339(next) {
            return now >= dt.with_timezone(&Utc);
        }
    }
    if job.last_run.is_none() {
        return true;
    }
    if let Some(last) = &job.last_run {
        if let Ok(dt) = DateTime::parse_from_rfc3339(last) {
            let elapsed = now.signed_duration_since(dt.with_timezone(&Utc));
            return elapsed.num_seconds() >= schedule_interval_secs(&job.schedule);
        }
    }
    true
}

fn schedule_interval_secs(schedule: &str) -> i64 {
    if let Some(rest) = schedule.strip_prefix("every:") {
        if let Some(secs) = rest.strip_suffix('s') {
            return secs.parse().unwrap_or(300);
        }
    }
    if schedule.contains("*/5m") || schedule == "@hourly" {
        return 300;
    }
    if schedule == "@daily" {
        return 86_400;
    }
    300
}

fn next_run_iso(schedule: &str, now: DateTime<Utc>) -> String {
    let secs = schedule_interval_secs(schedule);
    (now + chrono::Duration::seconds(secs)).to_rfc3339()
}