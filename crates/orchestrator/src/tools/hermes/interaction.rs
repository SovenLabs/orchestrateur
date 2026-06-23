//! `clarify` — port Hermess (question structurée à l'utilisateur).

use async_trait::async_trait;
use serde_json::{json, Value};

use super::json_result;
use crate::tools::tool::{Tool, ToolContext, ToolError, ToolResult};

const MAX_CHOICES: usize = 4;

pub struct ClarifyTool;

#[async_trait]
impl Tool for ClarifyTool {
    fn name(&self) -> &'static str {
        "clarify"
    }

    fn description(&self) -> &'static str {
        "Pose une question de clarification à l'utilisateur (jusqu'à 4 choix)."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"question":{"type":"string"},"choices":{"type":"array","items":{"type":"string"}}},"required":["question"]}"#
    }

    async fn execute(&self, _ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let question = args
            .get("question")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ToolError::InvalidArguments {
                tool: self.name().into(),
                message: "question requise".into(),
            })?;
        let choices: Vec<String> = args
            .get("choices")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| c.as_str().map(str::trim).filter(|s| !s.is_empty()))
                    .map(str::to_string)
                    .take(MAX_CHOICES)
                    .collect()
            })
            .unwrap_or_default();
        Ok(ToolResult {
            content: json_result(&json!({
                "status": "pending_user_input",
                "question": question,
                "choices": choices,
                "hint": "En mode headless, l'utilisateur doit répondre dans le prochain message.",
            })),
        })
    }
}