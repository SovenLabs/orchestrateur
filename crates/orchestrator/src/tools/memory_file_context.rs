use super::workspace_path::resolve_workspace_path;

use async_trait::async_trait;
use serde_json::Value;

use super::tool::{Tool, ToolContext, ToolError, ToolResult};

/// Lit le contexte d'un fichier workspace (MCP-style `file_context`).
pub struct MemoryFileContextTool;

const MAX_BYTES: usize = 32_768;

#[async_trait]
impl Tool for MemoryFileContextTool {
    fn name(&self) -> &'static str {
        "memory_file_context"
    }

    fn description(&self) -> &'static str {
        "Lit un extrait de fichier du workspace (chemins relatifs uniquement, max 32 KiB)."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"path":{"type":"string"},"max_chars":{"type":"integer"}},"required":["path"]}"#
    }

    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let rel = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments {
                tool: self.name().into(),
                message: "champ path requis".into(),
            })?;

        let max_chars = args
            .get("max_chars")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize)
            .unwrap_or(MAX_BYTES)
            .min(MAX_BYTES);

        let workspace = ctx.config().workspace_root.clone();
        let resolved = resolve_workspace_path(&workspace, rel).map_err(|message| {
            ToolError::InvalidArguments {
                tool: self.name().into(),
                message,
            }
        })?;

        let raw = tokio::fs::read_to_string(&resolved)
            .await
            .map_err(|e| ToolError::ExecutionFailed {
                tool: self.name().into(),
                message: format!("lecture {}: {e}", resolved.display()),
            })?;

        let excerpt: String = raw.chars().take(max_chars).collect();
        let truncated = raw.chars().count() > max_chars;

        Ok(ToolResult {
            content: format!(
                "# {}\n\n{}{}",
                rel,
                excerpt,
                if truncated { "\n\n[… tronqué]" } else { "" }
            ),
        })
    }
}

