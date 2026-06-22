use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::skills::SkillRegistry;
use crate::tool::{Tool, ToolContext, ToolError, ToolResult};

/// Liste les skills disponibles (builtin + hub + native).
pub struct SkillListTool {
    skills: Arc<SkillRegistry>,
}

impl SkillListTool {
    /// Crée l'outil avec le registre skills de la facade.
    #[must_use]
    pub fn new(skills: Arc<SkillRegistry>) -> Self {
        Self { skills }
    }
}

#[async_trait]
impl Tool for SkillListTool {
    fn name(&self) -> &'static str {
        "skill_list"
    }

    fn description(&self) -> &'static str {
        "Liste les skills enregistrées (builtin, hub, native) avec leur description."
    }

    fn parameters_schema(&self) -> &'static str {
        r#"{"type":"object","properties":{}}"#
    }

    async fn execute(&self, _ctx: &ToolContext, _args: &Value) -> Result<ToolResult, ToolError> {
        let lines: Vec<String> = self
            .skills
            .list()
            .into_iter()
            .map(|entry| {
                let version = entry
                    .version
                    .map(|v| format!(" v{v}"))
                    .unwrap_or_default();
                let source = match entry.source {
                    crate::skills::SkillSource::Builtin => "builtin",
                    crate::skills::SkillSource::Hub => "hub",
                    crate::skills::SkillSource::Native => "native",
                };
                format!(
                    "- {} [{source}{version}] — {}",
                    entry.name, entry.description
                )
            })
            .collect();
        Ok(ToolResult {
            content: if lines.is_empty() {
                "Aucune skill enregistrée.".into()
            } else {
                lines.join("\n")
            },
        })
    }
}