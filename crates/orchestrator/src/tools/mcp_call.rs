use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

use crate::mcp::McpError;
use super::tool::{Tool, ToolContext, ToolError, ToolResult};

/// Outil agent — appelle un outil MCP distant.
pub struct McpCallTool;

#[derive(Debug, Deserialize)]
struct McpCallArgs {
    server: String,
    tool: String,
    #[serde(default)]
    arguments: Value,
}

#[async_trait]
impl Tool for McpCallTool {
    fn name(&self) -> &'static str {
        "mcp_call"
    }

    fn description(&self) -> &'static str {
        "Appelle un outil MCP : paramètres server, tool, arguments (objet JSON)."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","required":["server","tool"],"properties":{"server":{"type":"string"},"tool":{"type":"string"},"arguments":{"type":"object"}}}"#
    }

    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let parsed: McpCallArgs = serde_json::from_value(args.clone()).map_err(|e| {
            ToolError::InvalidArguments {
                tool: self.name().into(),
                message: e.to_string(),
            }
        })?;
        let gateway = ctx.deps.mcp.as_ref().ok_or_else(|| ToolError::ExecutionFailed {
            tool: self.name().into(),
            message: McpError::Disabled.to_string(),
        })?;
        let result = gateway
            .call_tool(&parsed.server, &parsed.tool, parsed.arguments)
            .await
            .map_err(|e| ToolError::ExecutionFailed {
                tool: self.name().into(),
                message: e.to_string(),
            })?;
        Ok(ToolResult { content: result })
    }
}