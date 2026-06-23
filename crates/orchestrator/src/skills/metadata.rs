use serde::{Deserialize, Serialize};

use super::skill::SkillSource;

/// Catégorie fonctionnelle d'une skill (Phase 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SkillType {
    /// Skill générique (défaut).
    #[default]
    Generic,
    /// Étend le Cortex (hooks, outils mémoire).
    Cortex,
    /// Étend un agent persistant.
    Agent,
    /// Module ou signal B212.
    B212,
    /// Messagerie inter-agents.
    Communication,
}

impl SkillType {
    /// Parse une chaîne TOML / CLI.
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "cortex" => Self::Cortex,
            "agent" => Self::Agent,
            "b212" => Self::B212,
            "communication" | "comm" => Self::Communication,
            _ => Self::Generic,
        }
    }
}

/// Métadonnées complètes d'une skill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillMetadata {
    /// Identifiant stable.
    pub id: String,
    /// Nom affiché.
    pub name: String,
    /// Description lisible.
    pub description: String,
    /// Version semver libre.
    pub version: String,
    /// Auteur optionnel.
    pub author: Option<String>,
    /// Type fonctionnel.
    pub skill_type: SkillType,
    /// Dépendances sur d'autres skills (ids).
    pub dependencies: Vec<String>,
    /// Agents cibles (vide = global).
    pub agent_ids: Vec<String>,
    /// Origine dans le registre.
    pub source: SkillSource,
}

impl SkillMetadata {
    /// Métadonnées minimales pour une skill builtin.
    #[must_use]
    pub fn minimal(id: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            name: id.to_string(),
            description: description.to_string(),
            version: "builtin".into(),
            author: Some("orchestrateur".into()),
            skill_type: SkillType::Generic,
            dependencies: Vec::new(),
            agent_ids: Vec::new(),
            source: SkillSource::Builtin,
        }
    }

    /// Construit depuis un manifeste hub.
    #[must_use]
    pub fn from_manifest(
        id: &str,
        name: &str,
        description: &str,
        version: &str,
        author: Option<String>,
        skill_type: SkillType,
        dependencies: Vec<String>,
        agent_ids: Vec<String>,
        source: SkillSource,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            version: version.to_string(),
            author,
            skill_type,
            dependencies,
            agent_ids,
            source,
        }
    }
}