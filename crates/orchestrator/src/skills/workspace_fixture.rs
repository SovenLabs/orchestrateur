//! Validation du workspace de dev (`workspace/skills/`) contre les schémas P6.

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::config::OrchestratorConfig;
    use crate::skills::manifest::load_manifest;
    use crate::skills::marketplace::SkillsMarketplace;

    fn dev_workspace_root() -> Option<PathBuf> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../workspace");
        root.is_dir().then_some(root)
    }

    fn skip_without_workspace() -> PathBuf {
        dev_workspace_root().expect("workspace de dev absent — test ignoré en contexte isolé")
    }

    #[test]
    fn dev_marketplace_catalog_conforms_to_schema_v1() {
        let Some(workspace) = dev_workspace_root() else {
            return;
        };
        let mut config = OrchestratorConfig::default();
        config.workspace_root = workspace;

        let catalog = SkillsMarketplace::load_catalog(&config)
            .expect("catalog.json doit parser et respecter marketplace_require_signature");

        assert_eq!(
            catalog.version, 1,
            "catalog.json version doit être 1 (schéma documenté dans docs/skills-schema.md)"
        );

        for entry in &catalog.skills {
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().join("skill.toml");
            std::fs::write(&path, &entry.manifest_toml).expect("écriture manifeste temporaire");
            load_manifest(&path)
                .unwrap_or_else(|e| panic!("manifest_toml invalide pour `{}`: {e}", entry.id));
        }
    }

    #[test]
    fn dev_hub_skill_tomls_parse() {
        let workspace = skip_without_workspace();
        let hub = workspace.join("skills");
        if !hub.is_dir() {
            return;
        }

        let mut checked = 0usize;
        for entry in std::fs::read_dir(&hub).expect("lecture répertoire skills") {
            let entry = entry.expect("entrée hub");
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let manifest = path.join("skill.toml");
            if manifest.is_file() {
                load_manifest(&manifest)
                    .unwrap_or_else(|e| panic!("{}: {e}", manifest.display()));
                checked += 1;
            }
        }
        assert!(
            checked >= 2,
            "au moins pong et pong-native attendus dans workspace/skills"
        );
    }

    #[test]
    fn dev_marketplace_catalog_path_matches_config_default() {
        let workspace = skip_without_workspace();
        let config = OrchestratorConfig::default();
        let expected = workspace.join(&config.skills_hub.marketplace_catalog);
        assert!(
            expected.is_file(),
            "catalogue manquant: {}",
            expected.display()
        );
    }

    fn _assert_skill_md_present(workspace: &Path, id: &str) {
        let skill_md = workspace.join("skills").join(id).join("SKILL.md");
        assert!(
            skill_md.is_file(),
            "SKILL.md attendu pour la skill markdown `{id}`"
        );
    }

    #[test]
    fn dev_markdown_skills_have_skill_md() {
        let workspace = skip_without_workspace();
        for id in ["cortex-capture", "cortex-lint", "esprit-review"] {
            _assert_skill_md_present(&workspace, id);
        }
    }
}