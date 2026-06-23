//! Tâches de fond d'un agent persistant.

use std::path::Path;

use tokio::fs;

use crate::communication::receive_messages;
use crate::deps::AppDependencies;
use crate::persistent::{AgentStatus, PersistentAgent, PersistentAgentError};

use super::{persist_lifecycle, AgentHeartbeat};

/// Résultat d'un cycle de tâches de fond.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundTaskReport {
    /// Nombre de messages en inbox.
    pub inbox_count: usize,
    /// Nombre de fichiers dans `tasks/`.
    pub pending_tasks: usize,
    /// Tâches déclarées dans heartbeat.md exécutées.
    pub executed: Vec<String>,
}

/// Exécute les tâches de fond : scan inbox, scan tasks/, statut `Background`.
pub async fn run_background_tasks(
    _deps: &AppDependencies,
    agent: &mut PersistentAgent,
) -> Result<BackgroundTaskReport, PersistentAgentError> {
    let mut heartbeat = AgentHeartbeat::load(&agent.root).await?;
    let inbox = receive_messages(&agent.root, false).await?;
    let pending_tasks = count_task_files(&agent.root.join("tasks")).await?;

    let mut executed = Vec::new();
    for task in &heartbeat.background_tasks {
        match task.as_str() {
            "check_inbox" => {
                executed.push(format!("check_inbox:{inbox_count_label}", inbox_count_label = inbox.len()));
            }
            "scan_tasks" => {
                executed.push(format!("scan_tasks:{pending_tasks}"));
            }
            other => executed.push(format!("noop:{other}")),
        }
    }

    persist_lifecycle(
        &agent.root,
        &mut agent.config,
        AgentStatus::Background,
        &mut heartbeat,
    )
    .await?;

    Ok(BackgroundTaskReport {
        inbox_count: inbox.len(),
        pending_tasks,
        executed,
    })
}

async fn count_task_files(tasks_dir: &Path) -> Result<usize, PersistentAgentError> {
    if !tasks_dir.exists() {
        return Ok(0);
    }
    let mut count = 0usize;
    let mut entries = fs::read_dir(tasks_dir).await.map_err(|e| {
        PersistentAgentError::Io(e.to_string())
    })?;
    while let Some(entry) = entries.next_entry().await.map_err(|e| {
        PersistentAgentError::Io(e.to_string())
    })? {
        if entry.path().is_file() {
            count += 1;
        }
    }
    Ok(count)
}