use async_trait::async_trait;
use serde_json::Value;

use crate::mcp::McpError;
use crate::tool::{Tool, ToolContext, ToolError, ToolResult};

/// Outil agent — liste les outils MCP disponibles.
pub struct McpListToolsTool;

#[async_trait]
impl Tool for McpListToolsTool {
    fn name(&self) -> &'static str {
        "mcp_list_tools"
    }

    fn description(&self) -> &'static str {
        "Liste les outils exposés par les serveurs MCP connectés (server, name, description)."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{}}"#
    }

    async fn execute(&self, ctx: &ToolContext, _args: &Value) -> Result<ToolResult, ToolError> {
        let gateway = ctx.deps.mcp.as_ref().ok_or_else(|| ToolError::ExecutionFailed {
            tool: self.name().into(),
            message: McpError::Disabled.to_string(),
        })?;
        let tools = gateway.list_tools().await.map_err(|e| ToolError::ExecutionFailed {
            tool: self.name().into(),
            message: e.to_string(),
        })?;
        if tools.is_empty() {
            return Ok(ToolResult {
                content: "Aucun outil MCP disponible.".into(),
            });
        }
        let mut lines = Vec::new();
        for tool in tools {
            lines.push(format!(
                "{} / {} — {}",
                tool.server, tool.name, tool.description
            ));
        }
        Ok(ToolResult {
            content: lines.join("\n"),
        })
    }
}