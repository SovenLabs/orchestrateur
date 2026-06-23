//! `read_file`, `write_file`, `patch`, `search_files` — port Hermess file_tools.

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use super::json_result;
use crate::tools::registry::ToolRegistry;
use crate::tools::tool::{Tool, ToolContext, ToolError, ToolResult};
use crate::tools::workspace_path::resolve_workspace_path;

const MAX_READ_CHARS: usize = 100_000;

pub fn register(registry: &mut ToolRegistry) {
    registry.register(Arc::new(ReadFileTool));
    registry.register(Arc::new(WriteFileTool));
    registry.register(Arc::new(PatchTool));
    registry.register(Arc::new(SearchFilesTool));
}

pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &'static str {
        "read_file"
    }

    fn description(&self) -> &'static str {
        "Lit un fichier du workspace avec pagination (offset/limit en lignes)."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"path":{"type":"string"},"offset":{"type":"integer"},"limit":{"type":"integer"}},"required":["path"]}"#
    }

    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let rel = arg_str(args, "path")?;
        let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let limit = args
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(200) as usize;
        let workspace = ctx.config().workspace_root.clone();
        let path = resolve_workspace_path(&workspace, &rel).map_err(|m| ToolError::InvalidArguments {
            tool: self.name().into(),
            message: m,
        })?;
        let raw = tokio::fs::read_to_string(&path).await.map_err(|e| ToolError::ExecutionFailed {
            tool: self.name().into(),
            message: format!("lecture {}: {e}", path.display()),
        })?;
        let lines: Vec<&str> = raw.lines().collect();
        let slice = lines
            .get(offset..offset.saturating_add(limit))
            .unwrap_or(&[]);
        let numbered: String = slice
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{:>6}|{}", offset + i + 1, line))
            .collect::<Vec<_>>()
            .join("\n");
        let truncated = raw.chars().count() > MAX_READ_CHARS;
        Ok(ToolResult {
            content: json_result(&json!({
                "path": rel,
                "offset": offset,
                "limit": limit,
                "total_lines": lines.len(),
                "content": numbered,
                "truncated": truncated,
            })),
        })
    }
}

pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &'static str {
        "write_file"
    }

    fn description(&self) -> &'static str {
        "Écrit ou remplace un fichier dans le workspace (parents créés si besoin)."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"]}"#
    }

    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let rel = arg_str(args, "path")?;
        let content = arg_str(args, "content")?;
        let workspace = ctx.config().workspace_root.clone();
        let path = resolve_workspace_path(&workspace, &rel).map_err(|m| ToolError::InvalidArguments {
            tool: self.name().into(),
            message: m,
        })?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| ToolError::ExecutionFailed {
                tool: self.name().into(),
                message: e.to_string(),
            })?;
        }
        tokio::fs::write(&path, &content).await.map_err(|e| ToolError::ExecutionFailed {
            tool: self.name().into(),
            message: e.to_string(),
        })?;
        Ok(ToolResult {
            content: json_result(&json!({"success": true, "path": rel, "bytes": content.len()})),
        })
    }
}

pub struct PatchTool;

#[async_trait]
impl Tool for PatchTool {
    fn name(&self) -> &'static str {
        "patch"
    }

    fn description(&self) -> &'static str {
        "Modification ciblée : mode replace (old_string/new_string) ou patch multi-fichier."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"mode":{"type":"string","enum":["replace","patch"]},"path":{"type":"string"},"old_string":{"type":"string"},"new_string":{"type":"string"},"replace_all":{"type":"boolean"},"patch":{"type":"string"}}}"#
    }

    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let mode = args
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("replace");
        match mode {
            "replace" => {
                let rel = arg_str(args, "path")?;
                let old = arg_str(args, "old_string")?;
                let new = args
                    .get("new_string")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let replace_all = args
                    .get("replace_all")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let workspace = ctx.config().workspace_root.clone();
                let path = resolve_workspace_path(&workspace, &rel).map_err(|m| {
                    ToolError::InvalidArguments {
                        tool: self.name().into(),
                        message: m,
                    }
                })?;
                let raw = tokio::fs::read_to_string(&path).await.map_err(|e| {
                    ToolError::ExecutionFailed {
                        tool: self.name().into(),
                        message: e.to_string(),
                    }
                })?;
                let updated = if replace_all {
                    raw.replace(&old, &new)
                } else {
                    raw.replacen(&old, &new, 1)
                };
                if updated == raw {
                    return Err(ToolError::ExecutionFailed {
                        tool: self.name().into(),
                        message: "old_string introuvable".into(),
                    });
                }
                tokio::fs::write(&path, &updated).await.map_err(|e| ToolError::ExecutionFailed {
                    tool: self.name().into(),
                    message: e.to_string(),
                })?;
                Ok(ToolResult {
                    content: json_result(&json!({"success": true, "path": rel, "mode": "replace"})),
                })
            }
            "patch" => Err(ToolError::ExecutionFailed {
                tool: self.name().into(),
                message: "mode patch V4A multi-fichier : utiliser replace pour l'instant".into(),
            }),
            other => Err(ToolError::InvalidArguments {
                tool: self.name().into(),
                message: format!("mode inconnu: {other}"),
            }),
        }
    }
}

pub struct SearchFilesTool;

#[async_trait]
impl Tool for SearchFilesTool {
    fn name(&self) -> &'static str {
        "search_files"
    }

    fn description(&self) -> &'static str {
        "Recherche dans les fichiers du workspace (ripgrep si disponible, sinon walkdir+contains)."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"pattern":{"type":"string"},"path":{"type":"string"},"file_glob":{"type":"string"},"limit":{"type":"integer"}},"required":["pattern"]}"#
    }

    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let pattern = arg_str(args, "pattern")?;
        let sub = args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");
        let limit = args
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(50) as usize;
        let workspace = ctx.config().workspace_root.clone();
        let base = resolve_workspace_path(&workspace, sub).map_err(|m| ToolError::InvalidArguments {
            tool: self.name().into(),
            message: m,
        })?;
        let matches =
            rg_search(&base, &pattern, limit).unwrap_or_else(|| walk_search(&base, &pattern, limit));
        Ok(ToolResult {
            content: json_result(&json!({
                "pattern": pattern,
                "path": sub,
                "matches": matches,
            })),
        })
    }
}

fn rg_search(base: &Path, pattern: &str, limit: usize) -> Option<Vec<serde_json::Value>> {
    let output = Command::new("rg")
        .args(["--json", "-m", "1", pattern, base.to_str()?])
        .output()
        .ok()?;
    if !output.status.success() && output.stdout.is_empty() {
        return None;
    }
    let mut hits = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            if v.get("type").and_then(|t| t.as_str()) == Some("match") {
                let path = v
                    .pointer("/data/path/text")
                    .and_then(|p| p.as_str())
                    .unwrap_or("");
                let line_no = v.pointer("/data/line_number").and_then(|n| n.as_u64());
                hits.push(json!({"path": path, "line": line_no}));
                if hits.len() >= limit {
                    break;
                }
            }
        }
    }
    Some(hits)
}

fn walk_search(base: &Path, pattern: &str, limit: usize) -> Vec<serde_json::Value> {
    let mut hits = Vec::new();
    let needle = pattern.to_lowercase();
    for entry in walkdir::WalkDir::new(base)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        if let Ok(raw) = std::fs::read_to_string(entry.path()) {
            for (i, line) in raw.lines().enumerate() {
                if line.to_lowercase().contains(&needle) {
                    hits.push(json!({
                        "path": entry.path().display().to_string(),
                        "line": i + 1,
                    }));
                    if hits.len() >= limit {
                        return hits;
                    }
                }
            }
        }
    }
    hits
}

fn arg_str(args: &Value, key: &str) -> Result<String, ToolError> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .ok_or_else(|| ToolError::InvalidArguments {
            tool: key.into(),
            message: format!("champ {key} requis"),
        })
}