use async_trait::async_trait;
use crate::use_cases::{AssimilateFromText, DEFAULT_ASSIMILATION_SYSTEM_PROMPT};
use serde_json::Value;

use crate::tool::{Tool, ToolContext, ToolError, ToolResult};

/// Assimile du texte dans le Cortex via le provider LLM.
pub struct MemoryAssimilateTool;

#[async_trait]
impl Tool for MemoryAssimilateTool {
    fn name(&self) -> &'static str {
        "memory_assimilate"
    }

    fn description(&self) -> &'static str {
        "Assimile un texte en nouveau souvenir structuré dans le Cortex (titre, tags, contenu, backlinks)."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"text":{"type":"string"},"tags":{"type":"array","items":{"type":"string"}}},"required":["text"]}"#
    }

    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let text = args
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments {
                tool: self.name().into(),
                message: "champ text requis".into(),
            })?;

        let tags: Vec<String> = args
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|t| t.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();

        let mut prompt = text.to_string();
        if !tags.is_empty() {
            prompt = format!(
                "Tags suggérés: {}\n\n{text}",
                tags.join(", ")
            );
        }

        let (memory, _) = AssimilateFromText::new(ctx.deps.clone())
            .execute(&prompt, Some(DEFAULT_ASSIMILATION_SYSTEM_PROMPT))
            .await
            .map_err(|e| ToolError::ExecutionFailed {
                tool: self.name().into(),
                message: e.to_string(),
            })?;

        Ok(ToolResult {
            content: format!(
                "Souvenir assimilé: [{}] {}",
                memory.id, memory.title
            ),
        })
    }
}