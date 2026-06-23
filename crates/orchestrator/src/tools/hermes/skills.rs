//! `skills_list`, `skill_view`, `skill_manage` — port Hermess skills_tool + skill_manager.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use super::json_result;
use crate::tools::registry::ToolRegistry;
use crate::tools::tool::{Tool, ToolContext, ToolError, ToolResult};
use crate::tools::workspace_path::resolve_workspace_path;

pub fn register(registry: &mut ToolRegistry) {
    registry.register(Arc::new(SkillsListTool));
    registry.register(Arc::new(SkillViewTool));
    registry.register(Arc::new(SkillManageTool));
}

fn skills_root(ctx: &ToolContext) -> PathBuf {
    ctx.config().skills_hub_dir()
}

fn discover_skill_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if !root.is_dir() {
        return dirs;
    }
    for entry in walkdir::WalkDir::new(root)
        .min_depth(1)
        .max_depth(3)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_name() == "SKILL.md" {
            if let Some(parent) = entry.path().parent() {
                dirs.push(parent.to_path_buf());
            }
        }
    }
    dirs.sort();
    dirs.dedup();
    dirs
}

fn skill_id_from_path(root: &Path, dir: &Path) -> String {
    dir.strip_prefix(root)
        .unwrap_or(dir)
        .to_string_lossy()
        .replace('\\', "/")
}

fn parse_frontmatter(raw: &str) -> (Option<String>, Option<String>) {
    let trimmed = raw.trim_start();
    if !trimmed.starts_with("---") {
        return (None, None);
    }
    let rest = trimmed.trim_start_matches("---").trim_start();
    let Some(end) = rest.find("\n---") else {
        return (None, None);
    };
    let block = &rest[..end];
    let mut name = None;
    let mut description = None;
    for line in block.lines() {
        if let Some(v) = line.strip_prefix("name:") {
            name = Some(v.trim().trim_matches('"').to_string());
        }
        if let Some(v) = line.strip_prefix("description:") {
            description = Some(v.trim().trim_matches('"').to_string());
        }
    }
    (name, description)
}

pub struct SkillsListTool;

#[async_trait]
impl Tool for SkillsListTool {
    fn name(&self) -> &'static str {
        "skills_list"
    }

    fn description(&self) -> &'static str {
        "Liste les skills Markdown (nom + description) — progressive disclosure tier 1."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"category":{"type":"string"}}}"#
    }

    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let category = args.get("category").and_then(|v| v.as_str());
        let root = skills_root(ctx);
        let mut skills = Vec::new();
        for dir in discover_skill_dirs(&root) {
            let id = skill_id_from_path(&root, &dir);
            if let Some(cat) = category {
                if !id.starts_with(cat) {
                    continue;
                }
            }
            let skill_md = dir.join("SKILL.md");
            let raw = tokio::fs::read_to_string(&skill_md).await.unwrap_or_default();
            let (name, description) = parse_frontmatter(&raw);
            skills.push(json!({
                "id": id,
                "name": name.unwrap_or_else(|| id.clone()),
                "description": description.unwrap_or_default(),
                "path": skill_md.display().to_string(),
            }));
        }
        Ok(ToolResult {
            content: json_result(&json!({"skills": skills, "count": skills.len()})),
        })
    }
}

pub struct SkillViewTool;

#[async_trait]
impl Tool for SkillViewTool {
    fn name(&self) -> &'static str {
        "skill_view"
    }

    fn description(&self) -> &'static str {
        "Charge le contenu complet d'une skill (SKILL.md ou fichier lié)."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"name":{"type":"string"},"file_path":{"type":"string"}},"required":["name"]}"#
    }

    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let name = arg_str(args, "name")?;
        let file_path = args.get("file_path").and_then(|v| v.as_str());
        let root = skills_root(ctx);
        let skill_dir = root.join(name.replace('/', std::path::MAIN_SEPARATOR_STR));
        let target = if let Some(rel) = file_path {
            resolve_workspace_path(&root, &format!("{name}/{rel}")).map_err(|m| {
                ToolError::InvalidArguments {
                    tool: self.name().into(),
                    message: m,
                }
            })?
        } else {
            skill_dir.join("SKILL.md")
        };
        let content = tokio::fs::read_to_string(&target).await.map_err(|e| {
            ToolError::ExecutionFailed {
                tool: self.name().into(),
                message: format!("{}: {e}", target.display()),
            }
        })?;
        Ok(ToolResult {
            content: json_result(&json!({
                "name": name,
                "file": target.display().to_string(),
                "content": content,
            })),
        })
    }
}

pub struct SkillManageTool;

#[async_trait]
impl Tool for SkillManageTool {
    fn name(&self) -> &'static str {
        "skill_manage"
    }

    fn description(&self) -> &'static str {
        "Crée, modifie ou supprime des skills Markdown dans workspace/skills (P6)."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{"action":{"type":"string","enum":["create","edit","patch","delete","write_file","remove_file"]},"name":{"type":"string"},"content":{"type":"string"},"old_string":{"type":"string"},"new_string":{"type":"string"},"file_path":{"type":"string"},"file_content":{"type":"string"}},"required":["action","name"]}"#
    }

    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError> {
        let action = arg_str(args, "action")?;
        let name = arg_str(args, "name")?;
        let root = skills_root(ctx);
        let skill_dir = root.join(name.replace('/', std::path::MAIN_SEPARATOR_STR));
        match action.as_str() {
            "create" | "edit" => {
                let content = arg_str(args, "content")?;
                tokio::fs::create_dir_all(&skill_dir).await.map_err(|e| exec_err(self.name(), e))?;
                tokio::fs::write(skill_dir.join("SKILL.md"), &content)
                    .await
                    .map_err(|e| exec_err(self.name(), e))?;
                Ok(ok_json(&json!({"action": action, "name": name})))
            }
            "patch" => {
                let path = skill_dir.join("SKILL.md");
                let old = arg_str(args, "old_string")?;
                let new = args
                    .get("new_string")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let raw = tokio::fs::read_to_string(&path).await.map_err(|e| exec_err(self.name(), e))?;
                let updated = raw.replacen(&old, &new, 1);
                if updated == raw {
                    return Err(ToolError::ExecutionFailed {
                        tool: self.name().into(),
                        message: "old_string introuvable".into(),
                    });
                }
                tokio::fs::write(&path, &updated).await.map_err(|e| exec_err(self.name(), e))?;
                Ok(ok_json(&json!({"action": "patch", "name": name})))
            }
            "delete" => {
                tokio::fs::remove_dir_all(&skill_dir).await.map_err(|e| exec_err(self.name(), e))?;
                Ok(ok_json(&json!({"action": "delete", "name": name})))
            }
            "write_file" => {
                let rel = arg_str(args, "file_path")?;
                let content = arg_str(args, "file_content")?;
                let path = resolve_workspace_path(&skill_dir, &rel).map_err(|m| {
                    ToolError::InvalidArguments {
                        tool: self.name().into(),
                        message: m,
                    }
                })?;
                if let Some(parent) = path.parent() {
                    tokio::fs::create_dir_all(parent).await.map_err(|e| exec_err(self.name(), e))?;
                }
                tokio::fs::write(&path, &content).await.map_err(|e| exec_err(self.name(), e))?;
                Ok(ok_json(&json!({"action": "write_file", "path": path.display().to_string()})))
            }
            "remove_file" => {
                let rel = arg_str(args, "file_path")?;
                let path = resolve_workspace_path(&skill_dir, &rel).map_err(|m| {
                    ToolError::InvalidArguments {
                        tool: self.name().into(),
                        message: m,
                    }
                })?;
                tokio::fs::remove_file(&path).await.map_err(|e| exec_err(self.name(), e))?;
                Ok(ok_json(&json!({"action": "remove_file", "path": path.display().to_string()})))
            }
            other => Err(ToolError::InvalidArguments {
                tool: self.name().into(),
                message: format!("action inconnue: {other}"),
            }),
        }
    }
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

fn exec_err(tool: &str, e: impl std::fmt::Display) -> ToolError {
    ToolError::ExecutionFailed {
        tool: tool.into(),
        message: e.to_string(),
    }
}

fn ok_json(v: &Value) -> ToolResult {
    ToolResult {
        content: json_result(v),
    }
}