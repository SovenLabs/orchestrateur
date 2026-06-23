use std::path::Path;

use orchestrator::SkillType;

/// Type de plugin pour le template `skill.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillTemplateKind {
    /// Plugin subprocess.
    Subprocess,
    /// Plugin natif (DLL/SO).
    Native,
}

impl SkillTemplateKind {
    /// Parse depuis CLI.
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "native" => Self::Native,
            _ => Self::Subprocess,
        }
    }
}

/// Générateur de manifeste `skill.toml` (Phase 6).
#[derive(Debug, Clone)]
pub struct SkillManifestTemplate {
    /// Identifiant stable.
    pub id: String,
    /// Description.
    pub description: String,
    /// Type fonctionnel.
    pub skill_type: SkillType,
    /// Type plugin.
    pub plugin_kind: SkillTemplateKind,
    /// Auteur.
    pub author: Option<String>,
    /// Dépendances.
    pub dependencies: Vec<String>,
}

impl SkillManifestTemplate {
    /// Génère le contenu TOML et le corps `SKILL.md` associé.
    #[must_use]
    pub fn render(&self) -> (String, String) {
        let author_line = self
            .author
            .as_ref()
            .map(|a| format!("author = \"{a}\"\n"))
            .unwrap_or_default();
        let deps_line = if self.dependencies.is_empty() {
            String::new()
        } else {
            let joined = self
                .dependencies
                .iter()
                .map(|d| format!("\"{d}\""))
                .collect::<Vec<_>>()
                .join(", ");
            format!("dependencies = [{joined}]\n")
        };
        let skill_type = match self.skill_type {
            SkillType::Cortex => "cortex",
            SkillType::Agent => "agent",
            SkillType::B212 => "b212",
            SkillType::Communication => "communication",
            SkillType::Generic => "generic",
        };

        let plugin_section = match self.plugin_kind {
            SkillTemplateKind::Subprocess => {
                let cmd = if cfg!(windows) {
                    ("cmd", r#"["/c", "echo", "hello"]"#)
                } else {
                    ("echo", r#"["hello"]"#)
                };
                format!(
                    "\n[subprocess]\ncommand = \"{}\"\nargs = {}\nstdin_json = true\ntimeout_secs = 30\n",
                    cmd.0, cmd.1
                )
            }
            SkillTemplateKind::Native => {
                let lib = if cfg!(windows) {
                    "plugin.dll"
                } else {
                    "libplugin.so"
                };
                format!("\n[native]\nlibrary = \"{lib}\"\n")
            }
        };

        let toml = format!(
            "[skill]\nid = \"{}\"\nname = \"{}\"\ndescription = \"{}\"\nversion = \"0.1.0\"\n{author_line}skill_type = \"{skill_type}\"\n{deps_line}enabled = true\nkind = \"{}\"\n{plugin_section}",
            self.id,
            self.id,
            self.description,
            match self.plugin_kind {
                SkillTemplateKind::Subprocess => "subprocess",
                SkillTemplateKind::Native => "native",
            }
        );

        let markdown = format!(
            "# Skill — {}\n\n{}\n\n## Usage\n\n- Hub : `orch skill run {}`\n- Agent : routage via `skills_list` / `skill_view`\n",
            self.id, self.description, self.id
        );

        (toml, markdown)
    }

    /// Écrit le skill dans `hub_dir/{id}/`.
    ///
    /// # Errors
    ///
    /// Retourne une erreur IO si l'écriture échoue.
    pub fn write_to(&self, hub_dir: &Path) -> std::io::Result<()> {
        let root = hub_dir.join(&self.id);
        std::fs::create_dir_all(&root)?;
        let (toml, md) = self.render();
        std::fs::write(root.join("skill.toml"), toml)?;
        std::fs::write(root.join("SKILL.md"), md)?;
        Ok(())
    }
}