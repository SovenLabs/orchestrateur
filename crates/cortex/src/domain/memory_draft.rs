//! Brouillon de mémoire — point d'entrée unique avant matérialisation [`Memory`].
//!
//! Vit dans le **domaine pur** : toute persistance doit passer par validation
//! ([`crate::services::MemoryDraftValidator`]) puis [`MemoryDraft::into_memory`].

use serde::{Deserialize, Serialize};

use super::{Backlink, BacklinkKind, CortexError, Memory, MemoryId, MemoryKind, Tag};

/// Candidat de backlink tel que renvoyé par un LLM (avant validation domaine).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BacklinkDraft {
    /// Identifiant cible (UUID string) avant parsing domaine.
    pub target: String,
    /// Score de similarité ou de confiance (0.0–1.0).
    pub score: f32,
    /// Type de lien (sémantique ou wikilink explicite).
    #[serde(default)]
    pub kind: BacklinkDraftKind,
}

/// Type de backlink dans le brouillon (mappé vers [`BacklinkKind`] après validation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BacklinkDraftKind {
    /// Lien sémantique calculé par similarité.
    #[default]
    Semantic,
    /// Lien explicite `[[uuid]]` dans le contenu.
    ExplicitWikilink,
}

/// Brouillon structuré — entrée des providers IA et du bridge `Assimilate`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryDraft {
    /// Titre du souvenir candidat.
    pub title: String,
    /// Contenu Markdown brut.
    pub content: String,
    /// Tags sous forme de chaînes (normalisés à la conversion).
    #[serde(default)]
    pub tags: Vec<String>,
    /// Backlinks candidats issus du LLM.
    #[serde(default)]
    pub backlinks: Vec<BacklinkDraft>,
    /// Type sémantique du brouillon.
    #[serde(default)]
    pub kind: MemoryKind,
    /// Champs structurés optionnels selon le kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structured: Option<serde_json::Value>,
    /// Fichiers sources ayant motivé le brouillon (chemins relatifs workspace).
    #[serde(default)]
    pub source_files: Vec<String>,
}

impl MemoryDraft {
    /// Crée un brouillon minimal avec kind `context` par défaut.
    #[must_use]
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            tags: Vec::new(),
            backlinks: Vec::new(),
            kind: MemoryKind::default(),
            structured: None,
            source_files: Vec::new(),
        }
    }

    /// Valide structurellement et matérialise une entité domaine [`Memory`].
    ///
    /// N'effectue **pas** la validation adversariale — appeler
    /// [`crate::services::MemoryDraftValidator::validate`] avant.
    ///
    /// # Errors
    ///
    /// Propage une [`CortexError`] si le titre, le contenu, les tags ou les backlinks sont invalides.
    pub fn into_memory(self) -> Result<Memory, CortexError> {
        let tags: Vec<Tag> = self
            .tags
            .into_iter()
            .map(Tag::new)
            .collect::<Result<_, _>>()?;

        let mut memory = Memory::new(self.title, self.content)?;
        memory.kind = self.kind;
        memory.structured = self.structured;
        for tag in tags {
            memory.add_tag(tag);
        }

        let backlinks = self
            .backlinks
            .into_iter()
            .map(BacklinkDraft::into_backlink)
            .collect::<Result<Vec<_>, _>>()?;

        memory.set_backlinks(backlinks);
        Ok(memory)
    }
}

impl BacklinkDraft {
    fn into_backlink(self) -> Result<Backlink, CortexError> {
        let target: MemoryId = self.target.parse()?;
        let kind = match self.kind {
            BacklinkDraftKind::Semantic => BacklinkKind::Semantic,
            BacklinkDraftKind::ExplicitWikilink => BacklinkKind::ExplicitWikilink,
        };
        Backlink::new(target, self.score, kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draft_converts_to_valid_memory() {
        let id = MemoryId::new();
        let mut draft = MemoryDraft::new("Décision architecture", "Contenu du souvenir.");
        draft.tags = vec!["architecture".into(), "rust".into()];
        draft.kind = MemoryKind::Decision;
        draft.backlinks = vec![BacklinkDraft {
            target: id.to_string(),
            score: 0.87,
            kind: BacklinkDraftKind::Semantic,
        }];

        let memory = draft.into_memory().unwrap();
        assert_eq!(memory.title, "Décision architecture");
        assert_eq!(memory.tags.len(), 2);
        assert_eq!(memory.backlinks.len(), 1);
    }

    #[test]
    fn rejects_invalid_tag_in_draft() {
        let mut draft = MemoryDraft::new("T", "C");
        draft.tags = vec!["bad tag".into()];
        assert!(draft.into_memory().is_err());
    }

    #[test]
    fn serde_roundtrip_draft() {
        let mut draft = MemoryDraft::new("T", "C");
        draft.tags = vec!["rust".into()];
        let json = serde_json::to_string(&draft).unwrap();
        let parsed: MemoryDraft = serde_json::from_str(&json).unwrap();
        assert_eq!(draft, parsed);
    }
}