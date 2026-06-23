//! `terminal` et `execute_code`.

use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::process::Command;

use super::json_result;
use crate::tools::registry::ToolRegistry;
use crate::tools::tool::{Tool, ToolContext, ToolError, ToolResult};
use crate::tools::workspace_path::resolve_workspace_path;

pub fn register(registry: &mut ToolRegistry) {
    registry.register(Arc::new(TerminalTool));
    registry.register(Arc::new(ExecuteCodeTool));
}

pub struct TerminalTool;

#[async_trait]
impl Tool for TerminalTool {
    fn name(&self) -> &'static str {
        "terminal"
    }

    fn description(&self) -> &'static str {
        "Exécute une commande shell dans le workspace (timeout configurable)."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"command":{"type":"string"},"workdir":{"type":"string"},"timeout":{"type":"integer"}},"required":["command"]}"#
    }

    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let command = arg_str(args, "command")?;
        let timeout_secs = args
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(120)
            .min(600);
        let workspace = ctx.config().workspace_root.clone();
        let workdir = if let Some(wd) = args.get("workdir").and_then(|v| v.as_str()) {
            resolve_workspace_path(&workspace, wd).map_err(|m| ToolError::InvalidArguments {
                tool: self.name().into(),
                message: m,
            })?
        } else {
            workspace
        };
        let output = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            Command::new("cmd")
                .args(["/C", &command])
                .current_dir(&workdir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output(),
        )
        .await
        .map_err(|_| ToolError::ExecutionFailed {
            tool: self.name().into(),
            message: "timeout dépassé".into(),
        })?
        .map_err(|e| ToolError::ExecutionFailed {
            tool: self.name().into(),
            message: e.to_string(),
        })?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(ToolResult {
            content: json_result(&json!({
                "exit_code": output.status.code(),
                "stdout": stdout.chars().take(100_000).collect::<String>(),
                "stderr": stderr.chars().take(20_000).collect::<String>(),
            })),
        })
    }
}

pub struct ExecuteCodeTool;

#[async_trait]
impl Tool for ExecuteCodeTool {
    fn name(&self) -> &'static str {
        "execute_code"
    }

    fn description(&self) -> &'static str {
        "Exécute du code Python dans un subprocess isolé (stdout/stderr capturés)."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"code":{"type":"string"}},"required":["code"]}"#
    }

    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let code = arg_str(args, "code")?;
        let dir = ctx.config().workspace_root.join(".orchestrateur").join("sandbox");
        tokio::fs::create_dir_all(&dir).await.map_err(|e| exec_err(self.name(), e))?;
        let script = dir.join("snippet.py");
        tokio::fs::write(&script, &code).await.map_err(|e| exec_err(self.name(), e))?;
        let output = tokio::time::timeout(
            Duration::from_secs(300),
            Command::new("python")
                .arg(&script)
                .current_dir(&dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output(),
        )
        .await
        .map_err(|_| ToolError::ExecutionFailed {
            tool: self.name().into(),
            message: "timeout Python".into(),
        })?
        .map_err(|e| ToolError::ExecutionFailed {
            tool: self.name().into(),
            message: e.to_string(),
        })?;
        Ok(ToolResult {
            content: json_result(&json!({
                "exit_code": output.status.code(),
                "stdout": String::from_utf8_lossy(&output.stdout).chars().take(50_000).collect::<String>(),
                "stderr": String::from_utf8_lossy(&output.stderr).chars().take(10_000).collect::<String>(),
            })),
        })
    }
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