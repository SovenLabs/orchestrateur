//! Tests hub skills + plugins subprocess Phase 11.

use std::io::Write;

use orchestrator::config::OrchestratorConfig;
use orchestrator::skills::{SkillRegistry, SkillsHub};
use orchestrator::testing::MockBundle;

fn write_pong_manifest(dir: &std::path::Path) {
    let skill_dir = dir.join("skills").join("pong");
    std::fs::create_dir_all(&skill_dir).unwrap();
    let path = skill_dir.join("skill.toml");
    let mut file = std::fs::File::create(&path).unwrap();
    #[cfg(windows)]
    let toml = r#"
[skill]
id = "pong"
description = "Plugin pong"
version = "0.1.0"

[subprocess]
command = "cmd"
args = ["/c", "echo", "pong"]
"#;
    #[cfg(not(windows))]
    let toml = r#"
[skill]
id = "pong"
description = "Plugin pong"
version = "0.1.0"

[subprocess]
command = "echo"
args = ["pong"]
"#;
    write!(file, "{toml}").unwrap();
}

#[test]
fn skills_hub_discovers_filesystem_manifest() {
    let dir = tempfile::tempdir().unwrap();
    write_pong_manifest(dir.path());

    let mut config = OrchestratorConfig::default();
    config.workspace_root = dir.path().to_path_buf();
    config.skills_hub.enabled = true;

    let entries = SkillsHub::discover(&config).unwrap();
    assert!(entries.iter().any(|e| e.id == "pong"));
    assert_eq!(
        entries.iter().find(|e| e.id == "pong").unwrap().origin,
        "filesystem"
    );
}

#[tokio::test]
async fn skills_hub_loads_and_executes_subprocess_plugin() {
    let dir = tempfile::tempdir().unwrap();
    write_pong_manifest(dir.path());

    let mut bundle = MockBundle::new();
    bundle.config.workspace_root = dir.path().to_path_buf();
    bundle.config.skills_hub.enabled = true;
    bundle.config.skills_hub.auto_load = true;

    let registry = SkillRegistry::with_operational_skills_and_hub(bundle.into_deps());
    let out = registry
        .execute("pong", &orchestrator::SkillContext::default())
        .await
        .expect("plugin pong");
    assert_eq!(out.message, "pong");
}