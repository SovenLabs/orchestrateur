use std::str::FromStr;

use async_trait::async_trait;
use cortex::MemoryId;
use serde_json::Value;

use super::tool::{Tool, ToolContext, ToolError, ToolResult};

/// Récupère une mémoire complète par identifiant UUID.
pub struct MemoryGetTool;

#[async_trait]
impl Tool for MemoryGetTool {
    fn name(&self) -> &'static str {
        "memory_get"
    }

    fn description(&self) -> &'static str {
        "Récupère le détail complet d'un souvenir par son identifiant UUID v7."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"]}"#
    }

    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let id_str = args
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments {
                tool: self.name().into(),
                message: "champ id requis".into(),
            })?;

        let memory_id = MemoryId::from_str(id_str).map_err(|e| ToolError::InvalidArguments {
            tool: self.name().into(),
            message: e.to_string(),
        })?;

        let memory = ctx
            .memory_repo()
            .get_by_id(memory_id)
            .await
            .map_err(|e| ToolError::ExecutionFailed {
                tool: self.name().into(),
                message: e.to_string(),
            })?;

        Ok(ToolResult {
            content: format!(
                "# {}\n\nTags: {}\n\n{}",
                memory.title,
                memory
                    .tags
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
                memory.content
            ),
        })
    }
}