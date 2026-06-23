use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::skills::{SkillContext, SkillRegistry};
use super::tool::{Tool, ToolContext, ToolError, ToolResult};

/// Exécute une skill par identifiant (agentic Phase 12).
pub struct SkillExecuteTool {
    skills: Arc<SkillRegistry>,
}

impl SkillExecuteTool {
    /// Crée l'outil avec le registre skills de la facade.
    #[must_use]
    pub fn new(skills: Arc<SkillRegistry>) -> Self {
        Self { skills }
    }
}

#[async_trait]
impl Tool for SkillExecuteTool {
    fn name(&self) -> &'static str {
        "skill_execute"
    }

    fn description(&self) -> &'static str {
        "Exécute une skill par son nom. Paramètres : name (requis), query, text, tags, limit."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"name":{"type":"string"},"query":{"type":"string"},"text":{"type":"string"},"tags":{"type":"array","items":{"type":"string"}},"limit":{"type":"integer"}},"required":["name"]}"#
    }

    async fn execute(&self, _ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments {
                tool: self.name().into(),
                message: "champ name requis".into(),
            })?;
        let skill_ctx = SkillContext {
            query: args
                .get("query")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            text: args
                .get("text")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            tags: args
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default(),
            limit: args
                .get("limit")
                .and_then(serde_json::Value::as_u64)
                .map(|v| v as usize),
        };
        let output = self
            .skills
            .execute(name, &skill_ctx)
            .await
            .map_err(|e| ToolError::ExecutionFailed {
                tool: self.name().into(),
                message: e.to_string(),
            })?;
        Ok(ToolResult {
            content: output.message,
        })
    }
}