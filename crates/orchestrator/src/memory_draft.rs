//! Brouillon de mémoire issu des providers IA (xAI Structured Outputs, Ollama).
//!
//! Couche intermédiaire **orchestrator** — le domaine [`cortex::Memory`] n'est
//! jamais construit directement depuis le JSON externe.

use cortex::{Backlink, BacklinkKind, CortexError, Memory, MemoryId, Tag};
use serde::{Deserialize, Serialize};

/// Candidat de backlink tel que renvoyé par un LLM (avant validation domaine).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BacklinkDraft {
    pub target: String,
    pub score: f32,
    #[serde(default)]
    pub kind: BacklinkDraftKind,
}

/// Type de backlink dans le brouillon (mappé vers [`BacklinkKind`] après validation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BacklinkDraftKind {
    #[default]
    Semantic,
    ExplicitWikilink,
}

/// Brouillon structuré — point d'entrée unique pour xAI JSON Schema / Ollama.
///
/// Flux Phase 4 :
/// `xAI JSON → MemoryDraft → validation → Memory (cortex)`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryDraft {
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub backlinks: Vec<BacklinkDraft>,
}

impl MemoryDraft {
    /// Valide et matérialise une entité domaine [`Memory`].
    pub fn into_memory(self) -> Result<Memory, CortexError> {
        let tags: Vec<Tag> = self
            .tags
            .into_iter()
            .map(|t| Tag::new(t))
            .collect::<Result<_, _>>()?;

        let mut memory = Memory::new(self.title, self.content)?;
        for tag in tags {
            memory.add_tag(tag);
        }

        let backlinks = self
            .backlinks
            .into_iter()
            .map(|draft| draft.into_backlink())
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