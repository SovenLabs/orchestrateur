//! Brouillon de mémoire — point d'entrée unique avant matérialisation [`Memory`].
//!
//! Vit dans le **domaine pur** : toute persistance doit passer par validation
//! ([`crate::services::MemoryDraftValidator`]) puis [`MemoryDraft::into_memory`].

use serde::{Deserialize, Serialize};

use super::{Backlink, BacklinkKind, CortexError, Memory, MemoryId, Tag};

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
}

impl MemoryDraft {
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
        let draft = MemoryDraft {
            title: "Décision architecture".into(),
            content: "Contenu du souvenir.".into(),
            tags: vec!["architecture".into(), "rust".into()],
            backlinks: vec![BacklinkDraft {
                target: id.to_string(),
                score: 0.87,
                kind: BacklinkDraftKind::Semantic,
            }],
        };

        let memory = draft.into_memory().unwrap();
        assert_eq!(memory.title, "Décision architecture");
        assert_eq!(memory.tags.len(), 2);
        assert_eq!(memory.backlinks.len(), 1);
    }

    #[test]
    fn rejects_invalid_tag_in_draft() {
        let draft = MemoryDraft {
            title: "T".into(),
            content: "C".into(),
            tags: vec!["bad tag".into()],
            backlinks: vec![],
        };
        assert!(draft.into_memory().is_err());
    }

    #[test]
    fn serde_roundtrip_draft() {
        let draft = MemoryDraft {
            title: "T".into(),
            content: "C".into(),
            tags: vec!["rust".into()],
            backlinks: vec![],
        };
        let json = serde_json::to_string(&draft).unwrap();
        let parsed: MemoryDraft = serde_json::from_str(&json).unwrap();
        assert_eq!(draft, parsed);
    }
}