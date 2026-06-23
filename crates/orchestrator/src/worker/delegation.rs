//! Drain de la file de délégations (`.orchestrateur/delegations/`).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use crate::facade::OrchestratorFacade;
use crate::persistent::PersistentAgentError;

/// Enregistrement de délégation sur disque.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationRecord {
    /// Identifiant unique.
    pub id: String,
    /// Objectif de la tâche.
    pub goal: String,
    /// Contexte additionnel.
    pub context: String,
    /// Statut (`queued`, `running`, `done`, `failed`).
    pub status: String,
    /// Horodatage création.
    pub created_at: String,
    /// Agent cible optionnel.
    #[serde(default)]
    pub agent_id: Option<String>,
    /// Résultat ou erreur.
    #[serde(default)]
    pub result: Option<String>,
}

/// Nombre de délégations traitées lors du dernier tick.
#[derive(Debug, Clone, Default)]
pub struct DelegationDrainReport {
    /// Délégations complétées.
    pub completed: usize,
    /// Délégations en échec.
    pub failed: usize,
}

pub fn delegations_dir(workspace: &Path) -> PathBuf {
    workspace
        .join(".orchestrateur")
        .join("delegations")
}

/// Traite les délégations en file d'attente.
pub async fn drain_delegations(
    facade: &OrchestratorFacade,
    workspace: &Path,
    default_agent_id: &str,
) -> Result<DelegationDrainReport, PersistentAgentError> {
    let dir = delegations_dir(workspace);
    if !dir.exists() {
        return Ok(DelegationDrainReport::default());
    }

    let mut report = DelegationDrainReport::default();
    let mut entries = tokio::fs::read_dir(&dir).await.map_err(|e| {
        PersistentAgentError::Io(e.to_string())
    })?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| {
        PersistentAgentError::Io(e.to_string())
    })? {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let raw = tokio::fs::read_to_string(&path).await.map_err(|e| {
            PersistentAgentError::Io(e.to_string())
        })?;
        let mut record: DelegationRecord =
            serde_json::from_str(&raw).map_err(|e| PersistentAgentError::Config(e.to_string()))?;
        if record.status != "queued" {
            continue;
        }

        record.status = "running".into();
        write_record(&path, &record).await?;

        let agent_id = record
            .agent_id
            .as_deref()
            .unwrap_or(default_agent_id);
        let prompt = format!(
            "Délégation {} — objectif: {}\nContexte: {}",
            record.id, record.goal, record.context
        );

        match facade.agent_turn_for(agent_id, &prompt).await {
            Ok(result) => {
                record.status = "done".into();
                record.result = Some(result.reply);
                report.completed += 1;
            }
            Err(err) => {
                record.status = "failed".into();
                record.result = Some(err.to_string());
                report.failed += 1;
            }
        }
        write_record(&path, &record).await?;
    }

    Ok(report)
}

async fn write_record(path: &Path, record: &DelegationRecord) -> Result<(), PersistentAgentError> {
    let raw = serde_json::to_string_pretty(record)
        .map_err(|e| PersistentAgentError::Config(e.to_string()))?;
    tokio::fs::write(path, raw)
        .await
        .map_err(|e| PersistentAgentError::Io(e.to_string()))
}