use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::timeout;

use crate::config::SkillsHubEntryConfig;
use crate::error::SkillError;
use crate::skills::manifest::{SkillManifest, SkillPluginConfig, SubprocessPluginConfig};
use crate::skills::skill::{Skill, SkillContext, SkillOutput, SkillSource};

/// Skill chargée dynamiquement depuis le hub (subprocess).
pub struct SubprocessPluginSkill {
    id: String,
    description: String,
    version: String,
    config: SubprocessPluginConfig,
    working_dir: Option<PathBuf>,
}

impl SubprocessPluginSkill {
    /// Construit une skill depuis un manifeste hub subprocess.
    #[must_use]
    pub fn from_manifest(manifest: SkillManifest) -> Self {
        let config = match manifest.plugin {
            SkillPluginConfig::Subprocess(cfg) => cfg,
            SkillPluginConfig::Native(_) => SubprocessPluginConfig::default(),
        };
        Self {
            id: manifest.id,
            description: manifest.description,
            version: manifest.version,
            config,
            working_dir: Some(manifest.root),
        }
    }

    /// Construit une skill depuis une entrée inline TOML.
    #[must_use]
    pub fn from_entry(entry: SkillsHubEntryConfig) -> Self {
        Self {
            id: entry.id,
            description: entry.description,
            version: "inline".into(),
            config: SubprocessPluginConfig {
                command: entry.command,
                args: entry.args,
                stdin_json: entry.stdin_json,
                timeout_secs: entry.timeout_secs,
            },
            working_dir: None,
        }
    }
}

#[async_trait]
impl Skill for SubprocessPluginSkill {
    fn name(&self) -> &str {
        &self.id
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn source(&self) -> SkillSource {
        SkillSource::Hub
    }

    fn version(&self) -> Option<&str> {
        Some(&self.version)
    }

    async fn execute(&self, ctx: &SkillContext) -> Result<SkillOutput, SkillError> {
        run_subprocess(&self.config, self.working_dir.as_deref(), ctx).await
    }
}

async fn run_subprocess(
    config: &SubprocessPluginConfig,
    working_dir: Option<&std::path::Path>,
    ctx: &SkillContext,
) -> Result<SkillOutput, SkillError> {
    let mut command = Command::new(&config.command);
    command
        .args(&config.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(dir) = working_dir {
        command.current_dir(dir);
    }

    let mut child = command
        .spawn()
        .map_err(|e| SkillError::ExecutionFailed(format!("spawn: {e}")))?;

    if config.stdin_json {
        let payload = serde_json::to_string(ctx)
            .map_err(|e| SkillError::ExecutionFailed(format!("json ctx: {e}")))?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(payload.as_bytes())
                .await
                .map_err(|e| SkillError::ExecutionFailed(format!("stdin: {e}")))?;
        }
    }

    let duration = Duration::from_secs(config.timeout_secs.max(1));
    let output = timeout(duration, child.wait_with_output())
        .await
        .map_err(|_| SkillError::ExecutionFailed("timeout subprocess".into()))?
        .map_err(|e| SkillError::ExecutionFailed(format!("wait: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SkillError::ExecutionFailed(format!(
            "exit {:?}: {stderr}",
            output.status.code()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if config.stdin_json {
        let parsed: serde_json::Value = serde_json::from_str(&stdout)
            .map_err(|e| SkillError::ExecutionFailed(format!("json stdout: {e}")))?;
        let message = parsed
            .get("message")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| {
                SkillError::ExecutionFailed("stdout JSON doit contenir {message}".into())
            })?;
        return Ok(SkillOutput { message });
    }

    Ok(SkillOutput { message: stdout })
}