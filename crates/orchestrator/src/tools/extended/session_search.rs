//! `session_search` — discovery / scroll / browse.

use async_trait::async_trait;
use cortex::TurnRole;
use serde_json::{json, Value};

use super::json_result;
use crate::tools::tool::{Tool, ToolContext, ToolError, ToolResult};

pub struct SessionSearchTool;

#[async_trait]
impl Tool for SessionSearchTool {
    fn name(&self) -> &'static str {
        "session_search"
    }

    fn description(&self) -> &'static str {
        "Recherche dans l'historique des sessions : query (discovery), session_id+around_message_id (scroll), ou vide (browse)."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"query":{"type":"string"},"limit":{"type":"integer"},"session_id":{"type":"string"},"around_message_id":{"type":"integer"},"window":{"type":"integer"},"role_filter":{"type":"string"}}}"#
    }

    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let query = args.get("query").and_then(|v| v.as_str()).map(str::trim);
        let session_id = args.get("session_id").and_then(|v| v.as_str()).map(str::trim);
        let limit = args
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        if let Some(sid) = session_id.filter(|s| !s.is_empty()) {
            return scroll_mode(ctx, sid, args).await;
        }
        if let Some(q) = query.filter(|s| !s.is_empty()) {
            return discovery_mode(ctx, q, limit).await;
        }
        browse_mode(ctx, limit).await
    }
}

async fn browse_mode(ctx: &ToolContext, limit: usize) -> Result<ToolResult, ToolError> {
    let sessions = ctx
        .session_repo()
        .list_recent_sessions(limit)
        .await
        .map_err(|e| ToolError::ExecutionFailed {
            tool: "session_search".into(),
            message: e.to_string(),
        })?;
    let rows: Vec<_> = sessions
        .into_iter()
        .map(|s| {
            json!({
                "session_id": s.key.as_str(),
                "turn_count": s.turn_count,
                "updated_at": s.updated_at.to_rfc3339(),
                "preview": s.preview,
            })
        })
        .collect();
    Ok(ToolResult {
        content: json_result(&json!({"mode": "browse", "sessions": rows})),
    })
}

async fn discovery_mode(
    ctx: &ToolContext,
    query: &str,
    limit: usize,
) -> Result<ToolResult, ToolError> {
    let hits = ctx
        .session_repo()
        .search_turns(query, limit)
        .await
        .map_err(|e| ToolError::ExecutionFailed {
            tool: "session_search".into(),
            message: e.to_string(),
        })?;
    let rows: Vec<_> = hits
        .into_iter()
        .map(|h| {
            json!({
                "session_id": h.key.as_str(),
                "turn_index": h.turn_index,
                "role": role_str(h.role),
                "snippet": h.snippet,
            })
        })
        .collect();
    Ok(ToolResult {
        content: json_result(&json!({"mode": "discovery", "query": query, "hits": rows})),
    })
}

async fn scroll_mode(
    ctx: &ToolContext,
    session_id: &str,
    args: &Value,
) -> Result<ToolResult, ToolError> {
    use cortex::SessionKey;
    let key = SessionKey::new(session_id).map_err(|e| ToolError::InvalidArguments {
        tool: "session_search".into(),
        message: e.to_string(),
    })?;
    let turns = ctx
        .session_repo()
        .list_turns(&key)
        .await
        .map_err(|e| ToolError::ExecutionFailed {
            tool: "session_search".into(),
            message: e.to_string(),
        })?;
    let anchor = args
        .get("around_message_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let window = args
        .get("window")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as usize;
    let start = anchor.saturating_sub(window);
    let end = (anchor + window + 1).min(turns.len());
    let slice = &turns[start..end];
    let messages: Vec<_> = slice
        .iter()
        .enumerate()
        .map(|(i, t)| {
            json!({
                "index": start + i,
                "role": role_str(t.role),
                "content": t.content,
                "anchor": start + i == anchor,
            })
        })
        .collect();
    Ok(ToolResult {
        content: json_result(&json!({
            "mode": "scroll",
            "session_id": session_id,
            "messages": messages,
        })),
    })
}

fn role_str(role: TurnRole) -> &'static str {
    match role {
        TurnRole::User => "user",
        TurnRole::Assistant => "assistant",
        TurnRole::Tool => "tool",
        TurnRole::System => "system",
    }
}