//! `todo` et `memory` — état agent persistant (port Hermess).

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::json_result;
use crate::tools::registry::ToolRegistry;
use crate::tools::tool::{Tool, ToolContext, ToolError, ToolResult};

const ENTRY_DELIMITER: &str = "\n§\n";
const MAX_TODO_ITEMS: usize = 256;

pub fn register(registry: &mut ToolRegistry) {
    registry.register(Arc::new(TodoTool));
    registry.register(Arc::new(AgentMemoryTool));
}

fn agent_state_dir(ctx: &ToolContext) -> PathBuf {
    ctx.config()
        .workspace_root
        .join(".orchestrateur")
        .join("agent")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TodoItem {
    id: String,
    content: String,
    status: String,
}

pub struct TodoTool;

#[async_trait]
impl Tool for TodoTool {
    fn name(&self) -> &'static str {
        "todo"
    }

    fn description(&self) -> &'static str {
        "Liste de tâches de session : fournir todos[] pour écrire, omettre pour lire."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"todos":{"type":"array","items":{"type":"object","properties":{"id":{"type":"string"},"content":{"type":"string"},"status":{"type":"string"}}}},"merge":{"type":"boolean"},"session_id":{"type":"string"}}}"#
    }

    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let session = args
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        let path = agent_state_dir(ctx).join("todos").join(format!("{session}.json"));
        if args.get("todos").is_none() {
            let items = load_todos(&path).await?;
            return Ok(ToolResult {
                content: json_result(&json!({"todos": items})),
            });
        }
        let merge = args.get("merge").and_then(|v| v.as_bool()).unwrap_or(false);
        let incoming: Vec<TodoItem> = serde_json::from_value(
            args.get("todos").cloned().unwrap_or(json!([])),
        )
        .map_err(|e| ToolError::InvalidArguments {
            tool: self.name().into(),
            message: e.to_string(),
        })?;
        let mut items = if merge {
            load_todos(&path).await?
        } else {
            Vec::new()
        };
        for t in incoming.into_iter().take(MAX_TODO_ITEMS) {
            if let Some(existing) = items.iter_mut().find(|i| i.id == t.id) {
                if !t.content.is_empty() {
                    existing.content = t.content;
                }
                if !t.status.is_empty() {
                    existing.status = t.status;
                }
            } else {
                items.push(t);
            }
        }
        save_todos(&path, &items).await?;
        Ok(ToolResult {
            content: json_result(&json!({"todos": items})),
        })
    }
}

async fn load_todos(path: &PathBuf) -> Result<Vec<TodoItem>, ToolError> {
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let raw = tokio::fs::read_to_string(path).await.map_err(|e| ToolError::ExecutionFailed {
        tool: "todo".into(),
        message: e.to_string(),
    })?;
    serde_json::from_str(&raw).map_err(|e| ToolError::ExecutionFailed {
        tool: "todo".into(),
        message: e.to_string(),
    })
}

async fn save_todos(path: &PathBuf, items: &[TodoItem]) -> Result<(), ToolError> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| ToolError::ExecutionFailed {
            tool: "todo".into(),
            message: e.to_string(),
        })?;
    }
    let raw = serde_json::to_string_pretty(items).map_err(|e| ToolError::ExecutionFailed {
        tool: "todo".into(),
        message: e.to_string(),
    })?;
    tokio::fs::write(path, raw).await.map_err(|e| ToolError::ExecutionFailed {
        tool: "todo".into(),
        message: e.to_string(),
    })
}

pub struct AgentMemoryTool;

#[async_trait]
impl Tool for AgentMemoryTool {
    fn name(&self) -> &'static str {
        "memory"
    }

    fn description(&self) -> &'static str {
        "Mémoire agent curatée (MEMORY.md / USER.md) — distincte des mémoires Cortex."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"target":{"type":"string","enum":["memory","user"]},"action":{"type":"string","enum":["add","replace","remove"]},"content":{"type":"string"},"old_text":{"type":"string"}},"required":["target","action"]}"#
    }

    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let target = arg_str(args, "target")?;
        let action = arg_str(args, "action")?;
        let filename = match target.as_str() {
            "memory" => "MEMORY.md",
            "user" => "USER.md",
            other => {
                return Err(ToolError::InvalidArguments {
                    tool: self.name().into(),
                    message: format!("target inconnu: {other}"),
                });
            }
        };
        let path = agent_state_dir(ctx).join(filename);
        tokio::fs::create_dir_all(agent_state_dir(ctx))
            .await
            .map_err(|e| exec_err(self.name(), e))?;
        let mut entries = load_entries(&path).await?;
        match action.as_str() {
            "add" => {
                let content = arg_str(args, "content")?;
                entries.push(content);
            }
            "replace" => {
                let old = arg_str(args, "old_text")?;
                let new = arg_str(args, "content")?;
                let pos = entries.iter().position(|e| e.contains(&old)).ok_or_else(|| {
                    ToolError::ExecutionFailed {
                        tool: self.name().into(),
                        message: "old_text introuvable".into(),
                    }
                })?;
                entries[pos] = new;
            }
            "remove" => {
                let old = arg_str(args, "old_text")?;
                entries.retain(|e| !e.contains(&old));
            }
            other => {
                return Err(ToolError::InvalidArguments {
                    tool: self.name().into(),
                    message: format!("action inconnue: {other}"),
                });
            }
        }
        save_entries(&path, &entries).await?;
        Ok(ToolResult {
            content: json_result(&json!({
                "target": target,
                "entries": entries.len(),
                "content": entries.join(ENTRY_DELIMITER),
            })),
        })
    }
}

async fn load_entries(path: &PathBuf) -> Result<Vec<String>, ToolError> {
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let raw = tokio::fs::read_to_string(path).await.map_err(|e| exec_err("memory", e))?;
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    Ok(raw.split(ENTRY_DELIMITER).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect())
}

async fn save_entries(path: &PathBuf, entries: &[String]) -> Result<(), ToolError> {
    let body = entries.join(ENTRY_DELIMITER);
    tokio::fs::write(path, body).await.map_err(|e| exec_err("memory", e))
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