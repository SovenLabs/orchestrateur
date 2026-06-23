//! `delegate_task` et `cronjob`.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::json_result;
use crate::tools::registry::ToolRegistry;
use crate::tools::tool::{Tool, ToolContext, ToolError, ToolResult};

pub fn register(registry: &mut ToolRegistry) {
    registry.register(Arc::new(DelegateTaskTool));
    registry.register(Arc::new(CronjobTool));
}

pub struct DelegateTaskTool;

#[async_trait]
impl Tool for DelegateTaskTool {
    fn name(&self) -> &'static str {
        "delegate_task"
    }

    fn description(&self) -> &'static str {
        "Délègue une tâche à un sous-agent (enregistrement + file d'attente locale)."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"goal":{"type":"string"},"context":{"type":"string"},"tasks":{"type":"array","items":{"type":"object"}}},"required":["goal"]}"#
    }

    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let goal = arg_str(args, "goal")?;
        let context = args
            .get("context")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let dir = delegation_dir(ctx);
        tokio::fs::create_dir_all(&dir).await.map_err(|e| exec_err(self.name(), e))?;
        let id = format!("deleg-{}", Utc::now().timestamp_millis());
        let record = json!({
            "id": id,
            "goal": goal,
            "context": context,
            "status": "queued",
            "created_at": Utc::now().to_rfc3339(),
        });
        let path = dir.join(format!("{id}.json"));
        tokio::fs::write(&path, serde_json::to_string_pretty(&record).unwrap())
            .await
            .map_err(|e| exec_err(self.name(), e))?;
        Ok(ToolResult {
            content: json_result(&json!({
                "status": "queued",
                "id": id,
                "message": "Tâche enregistrée — exécution parallèle via worker à brancher sur la facade.",
            })),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CronJob {
    id: String,
    name: String,
    prompt: String,
    schedule: String,
    enabled: bool,
    created_at: String,
}

pub struct CronjobTool;

#[async_trait]
impl Tool for CronjobTool {
    fn name(&self) -> &'static str {
        "cronjob"
    }

    fn description(&self) -> &'static str {
        "Planifie des tâches récurrentes ou différées (CRUD jobs JSON locaux)."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"action":{"type":"string","enum":["create","list","update","pause","resume","remove","run"]},"job_id":{"type":"string"},"name":{"type":"string"},"prompt":{"type":"string"},"schedule":{"type":"string"}},"required":["action"]}"#
    }

    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let action = arg_str(args, "action")?;
        let store = cron_store_path(ctx);
        let mut jobs = load_jobs(&store).await?;
        match action.as_str() {
            "list" => Ok(ToolResult {
                content: json_result(&json!({"jobs": jobs})),
            }),
            "create" => {
                let name = arg_str(args, "name")?;
                let prompt = arg_str(args, "prompt")?;
                let schedule = args
                    .get("schedule")
                    .and_then(|v| v.as_str())
                    .unwrap_or("@daily")
                    .to_string();
                let id = format!("job-{}", Utc::now().timestamp_millis());
                jobs.push(CronJob {
                    id: id.clone(),
                    name,
                    prompt,
                    schedule,
                    enabled: true,
                    created_at: Utc::now().to_rfc3339(),
                });
                save_jobs(&store, &jobs).await?;
                Ok(ok(&json!({"created": id})))
            }
            "remove" => {
                let id = arg_str(args, "job_id")?;
                jobs.retain(|j| j.id != id);
                save_jobs(&store, &jobs).await?;
                Ok(ok(&json!({"removed": id})))
            }
            "pause" => {
                toggle_job(&mut jobs, args, false)?;
                save_jobs(&store, &jobs).await?;
                Ok(ok(&json!({"paused": true})))
            }
            "resume" => {
                toggle_job(&mut jobs, args, true)?;
                save_jobs(&store, &jobs).await?;
                Ok(ok(&json!({"resumed": true})))
            }
            "update" | "run" => Ok(ToolResult {
                content: json_result(&json!({
                    "status": "not_implemented",
                    "action": action,
                    "hint": "Brancher sur le scheduler daemon (tick cron).",
                })),
            }),
            other => Err(ToolError::InvalidArguments {
                tool: self.name().into(),
                message: format!("action inconnue: {other}"),
            }),
        }
    }
}

fn toggle_job(jobs: &mut Vec<CronJob>, args: &Value, enabled: bool) -> Result<(), ToolError> {
    let id = arg_str(args, "job_id")?;
    if let Some(job) = jobs.iter_mut().find(|j| j.id == id) {
        job.enabled = enabled;
        Ok(())
    } else {
        Err(ToolError::ExecutionFailed {
            tool: "cronjob".into(),
            message: format!("job introuvable: {id}"),
        })
    }
}

fn delegation_dir(ctx: &ToolContext) -> PathBuf {
    ctx.config()
        .workspace_root
        .join(".orchestrateur")
        .join("delegations")
}

fn cron_store_path(ctx: &ToolContext) -> PathBuf {
    ctx.config()
        .workspace_root
        .join(".orchestrateur")
        .join("cron")
        .join("jobs.json")
}

async fn load_jobs(path: &PathBuf) -> Result<Vec<CronJob>, ToolError> {
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let raw = tokio::fs::read_to_string(path).await.map_err(|e| exec_err("cronjob", e))?;
    serde_json::from_str(&raw).map_err(|e| exec_err("cronjob", e))
}

async fn save_jobs(path: &PathBuf, jobs: &[CronJob]) -> Result<(), ToolError> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| exec_err("cronjob", e))?;
    }
    let raw = serde_json::to_string_pretty(jobs).map_err(|e| exec_err("cronjob", e))?;
    tokio::fs::write(path, raw).await.map_err(|e| exec_err("cronjob", e))
}

fn arg_str(args: &Value, key: &str) -> Result<String, ToolError> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .ok_or_else(|| ToolError::InvalidArguments {
            tool: key.into(),
            message: format!("champ {key} requis"),
        })
}

fn exec_err(tool: &str, e: impl std::fmt::Display) -> ToolError {
    ToolError::ExecutionFailed {
        tool: tool.into(),
        message: e.to_string(),
    }
}

fn ok(v: &Value) -> ToolResult {
    ToolResult {
        content: json_result(v),
    }
}