use async_trait::async_trait;
use cortex::SearchFilter;
use crate::use_cases::SearchMemories;
use serde_json::Value;

use crate::tool::{Tool, ToolContext, ToolError, ToolResult};

/// Recherche sémantique dans les mémoires Cortex.
pub struct MemorySearchTool;

#[async_trait]
impl Tool for MemorySearchTool {
    fn name(&self) -> &'static str {
        "memory_search"
    }

    fn description(&self) -> &'static str {
        "Recherche sémantique dans les souvenirs persistés du Cortex. Paramètre query (texte) et limit optionnel."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"query":{"type":"string"},"limit":{"type":"integer"}},"required":["query"]}"#
    }

    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
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

        let filter = SearchFilter {
            limit: Some(limit),
            ..SearchFilter::default()
        };

        let hits = SearchMemories::new(ctx.deps.clone())
            .execute(query, &filter)
            .await
            .map_err(|e| ToolError::ExecutionFailed {
                tool: self.name().into(),
                message: e.to_string(),
            })?;

        if hits.is_empty() {
            return Ok(ToolResult {
                content: "Aucun souvenir pertinent trouvé.".into(),
            });
        }

        let mut lines = Vec::with_capacity(hits.len());
        for hit in hits {
            let memory = ctx
                .memory_repo()
                .get_by_id(hit.memory_id)
                .await
                .map_err(|e| ToolError::ExecutionFailed {
                    tool: self.name().into(),
                    message: e.to_string(),
                })?;
            lines.push(format!(
                "- [{}] {} (score {:.2}): {}",
                memory.id,
                memory.title,
                hit.score,
                truncate(&memory.content, 200)
            ));
        }

        Ok(ToolResult {
            content: lines.join("\n"),
        })
    }
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max).collect();
    out.push_str("…");
    out
}