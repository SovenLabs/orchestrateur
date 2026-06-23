//! Tests Phase 6 — skills extensibles, dépendances, injection agent, templates.

use orchestrateur_plugins::{InstalledSkillRegistry, SkillManifestTemplate, SkillTemplateKind};
use orchestrator::{
    CortexExtensionRegistry, DependencyError, OrchestratorConfig, SkillLoader, SkillRegistry,
    SkillType,
};
use orchestrator::testing::MockBundle;

#[test]
fn dependency_order_loads_prerequisite_first() {
    let dir = tempfile::tempdir().unwrap();
    write_skill(
        &dir.path().join("skills").join("base"),
        "base",
        &[],
    );
    write_skill(
        &dir.path().join("skills").join("child"),
        "child",
        &["base"],
    );

    let mut config = OrchestratorConfig::default();
    config.workspace_root = dir.path().to_path_buf();
    config.skills_hub.enabled = true;

    let descriptors = SkillLoader::discover(&config).unwrap();
    let metadata = SkillLoader::collect_metadata(&descriptors);
    let order = orchestrator::resolve_load_order(&descriptors, &metadata).unwrap();
    assert!(order.iter().position(|id| id == "base").unwrap()
        < order.iter().position(|id| id == "child").unwrap());
}

#[test]
fn missing_dependency_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    write_skill(&dir.path().join("skills").join("lonely"), "lonely", &["missing"]);

    let mut config = OrchestratorConfig::default();
    config.workspace_root = dir.path().to_path_buf();
    config.skills_hub.enabled = true;

    let descriptors = SkillLoader::discover(&config).unwrap();
    let metadata = SkillLoader::collect_metadata(&descriptors);
    let err = orchestrator::resolve_load_order(&descriptors, &metadata).unwrap_err();
    assert!(matches!(err, DependencyError::Missing { .. }));
}

#[tokio::test]
async fn skill_loader_registers_hub_plugins() {
    let dir = tempfile::tempdir().unwrap();
    write_skill(&dir.path().join("skills").join("pong"), "pong", &[]);

    let mut bundle = MockBundle::new();
    bundle.config.workspace_root = dir.path().to_path_buf();
    bundle.config.skills_hub.enabled = true;
    bundle.config.skills_hub.auto_load = false;
    let config = bundle.config.clone();

    let mut registry = SkillRegistry::with_operational_skills(bundle.into_deps());
    let count = SkillLoader::load_into(&mut registry, &config).unwrap();
    assert_eq!(count, 1);
    let out = registry
        .execute("pong", &orchestrator::SkillContext::default())
        .await
        .unwrap();
    assert!(!out.message.is_empty());
}

#[test]
fn manifest_template_writes_skill_files() {
    let dir = tempfile::tempdir().unwrap();
    let template = SkillManifestTemplate {
        id: "demo-skill".into(),
        description: "Demo".into(),
        skill_type: SkillType::Cortex,
        plugin_kind: SkillTemplateKind::Subprocess,
        author: Some("tester".into()),
        dependencies: vec!["base".into()],
    };
    template.write_to(&dir.path().join("skills")).unwrap();
    let toml = std::fs::read_to_string(dir.path().join("skills/demo-skill/skill.toml")).unwrap();
    assert!(toml.contains("skill_type = \"cortex\""));
    assert!(std::fs::metadata(dir.path().join("skills/demo-skill/SKILL.md")).is_ok());
}

#[test]
fn installed_registry_lists_hub_skills() {
    let dir = tempfile::tempdir().unwrap();
    write_skill(&dir.path().join("skills").join("alpha"), "alpha", &[]);

    let mut config = OrchestratorConfig::default();
    config.workspace_root = dir.path().to_path_buf();
    config.skills_hub.enabled = true;

    let list = InstalledSkillRegistry::list(&config).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].descriptor.id, "alpha");
}

#[test]
fn cortex_extension_registry_applies_search_transform() {
    use std::sync::Arc;

    struct Prefix;
    impl orchestrator::CortexExtension for Prefix {
        fn name(&self) -> &str {
            "prefix"
        }
        fn transform_search_query(&self, query: &str) -> Option<String> {
            Some(format!("scoped:{query}"))
        }
    }

    let registry = CortexExtensionRegistry::new();
    registry.register(Arc::new(Prefix));
    assert_eq!(
        registry.apply_search_transforms("test".into()),
        "scoped:test"
    );
}

fn write_skill(root: &std::path::Path, id: &str, deps: &[&str]) {
    std::fs::create_dir_all(root).unwrap();
    let deps_line = if deps.is_empty() {
        String::new()
    } else {
        let joined = deps
            .iter()
            .map(|d| format!("\"{d}\""))
            .collect::<Vec<_>>()
            .join(", ");
        format!("dependencies = [{joined}]\n")
    };
    #[cfg(windows)]
    let subprocess = r#"
[subprocess]
command = "cmd"
args = ["/c", "echo", "ok"]
"#;
    #[cfg(not(windows))]
    let subprocess = r#"
[subprocess]
command = "echo"
args = ["ok"]
"#;
    let toml = format!(
        "[skill]\nid = \"{id}\"\ndescription = \"{id}\"\nversion = \"0.1.0\"\n{deps_line}{subprocess}"
    );
    std::fs::write(root.join("skill.toml"), toml).unwrap();
}