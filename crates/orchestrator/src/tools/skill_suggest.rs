use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::skills::marketplace::suggest_skills;
use crate::skills::SkillRegistry;
use crate::tool::{Tool, ToolContext, ToolError, ToolResult};

/// Suggère des skills pertinentes pour une requête utilisateur (Phase 13).
pub struct SkillSuggestTool {
    skills: Arc<SkillRegistry>,
}

impl SkillSuggestTool {
    /// Crée l'outil avec le registre skills de la facade.
    #[must_use]
    pub fn new(skills: Arc<SkillRegistry>) -> Self {
        Self { skills }
    }
}

#[async_trait]
impl Tool for SkillSuggestTool {
    fn name(&self) -> &'static str {
        "skill_suggest"
    }

    fn description(&self) -> &'static str {
        "Suggère des skills pertinentes pour une requête (correspondance nom/description)."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"query":{"type":"string"},"limit":{"type":"integer"}},"required":["query"]}"#
    }

    async fn execute(&self, _ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments {
                tool: self.name().into(),
                message: "champ query requis".into(),
            })?;
        let limit = args
            .get("limit")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(5) as usize;
        let hits = suggest_skills(&self.skills.list(), query, limit);
        if hits.is_empty() {
            return Ok(ToolResult {
                content: "Aucune skill suggérée pour cette requête.".into(),
            });
        }
        let lines: Vec<String> = hits
            .iter()
            .map(|entry| {
                format!(
                    "- {} — {}",
                    entry.name, entry.description
                )
            })
            .collect();
        Ok(ToolResult {
            content: format!(
                "Skills suggérées ({}) :\n{}",
                hits.len(),
                lines.join("\n")
            ),
        })
    }
}