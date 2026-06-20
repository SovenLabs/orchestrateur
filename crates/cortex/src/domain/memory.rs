use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{Backlink, CortexError, MemoryId, Tag};

/// Entité centrale : un souvenir persistant en Markdown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Memory {
    /// Identifiant unique (UUID v7).
    pub id: MemoryId,
    /// Titre lisible du souvenir.
    pub title: String,
    /// Corps Markdown brut (hors frontmatter).
    pub content: String,
    /// Tags normalisés associés au souvenir.
    pub tags: Vec<Tag>,
    /// Date de création UTC.
    pub created_at: DateTime<Utc>,
    /// Date de dernière modification UTC.
    pub updated_at: DateTime<Utc>,
    /// Liens sortants vers d'autres mémoires.
    pub backlinks: Vec<Backlink>,
}

impl Memory {
    /// Crée une nouvelle mémoire avec horodatage courant.
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Result<Self, CortexError> {
        let title = title.into();
        let content = content.into();
        if title.trim().is_empty() {
            return Err(CortexError::EmptyTitle);
        }
        if content.trim().is_empty() {
            return Err(CortexError::EmptyContent);
        }
        let now = Utc::now();
        Ok(Self {
            id: MemoryId::new(),
            title,
            content,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            backlinks: Vec::new(),
        })
    }

    /// Reconstruit une mémoire depuis ses champs (ex: parsing Markdown).
    pub fn reconstruct(
        id: MemoryId,
        title: String,
        content: String,
        tags: Vec<Tag>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        backlinks: Vec<Backlink>,
    ) -> Result<Self, CortexError> {
        if title.trim().is_empty() {
            return Err(CortexError::EmptyTitle);
        }
        if content.trim().is_empty() {
            return Err(CortexError::EmptyContent);
        }
        Ok(Self {
            id,
            title,
            content,
            tags,
            created_at,
            updated_at,
            backlinks,
        })
    }

    /// Met à jour le contenu et rafraîchit `updated_at`.
    pub fn update_content(&mut self, content: impl Into<String>) -> Result<(), CortexError> {
        let content = content.into();
        if content.trim().is_empty() {
            return Err(CortexError::EmptyContent);
        }
        self.content = content;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Ajoute un tag s'il n'existe pas déjà.
    pub fn add_tag(&mut self, tag: Tag) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// Remplace les backlinks (dédupliqués par cible) et met à jour l'horodatage.
    pub fn set_backlinks(&mut self, backlinks: Vec<Backlink>) {
        self.backlinks = dedupe_backlinks(backlinks);
        self.updated_at = Utc::now();
    }

    /// Ajoute ou met à jour un backlink (conserve le score le plus élevé par cible).
    pub fn add_or_update_backlink(&mut self, backlink: Backlink) {
        if let Some(existing) = self
            .backlinks
            .iter_mut()
            .find(|b| b.target == backlink.target)
        {
            if backlink.score > existing.score {
                *existing = backlink;
            }
        } else {
            self.backlinks.push(backlink);
        }
        self.updated_at = Utc::now();
    }

    /// Nombre de backlinks sortants.
    pub fn backlink_count(&self) -> usize {
        self.backlinks.len()
    }

    /// Indique si la mémoire possède ce tag.
    pub fn has_tag(&self, tag: &Tag) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

/// Déduplique les backlinks par `target` en conservant le score maximal.
pub(crate) fn dedupe_backlinks(backlinks: Vec<Backlink>) -> Vec<Backlink> {
    let mut out: Vec<Backlink> = Vec::new();
    for bl in backlinks {
        if let Some(existing) = out.iter_mut().find(|b| b.target == bl.target) {
            if bl.score > existing.score {
                *existing = bl;
            }
        } else {
            out.push(bl);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_memory_has_uuid_v7() {
        let mem = Memory::new("Titre", "Contenu").unwrap();
        assert_eq!(mem.id.as_uuid().get_version(), Some(uuid::Version::SortRand));
    }

    #[test]
    fn rejects_empty_title() {
        assert!(matches!(Memory::new("", "x"), Err(CortexError::EmptyTitle)));
    }

    #[test]
    fn rejects_empty_content() {
        assert!(matches!(Memory::new("t", ""), Err(CortexError::EmptyContent)));
    }

    #[test]
    fn add_tag_deduplicates() {
        let mut mem = Memory::new("T", "C").unwrap();
        let tag = Tag::new("rust").unwrap();
        mem.add_tag(tag.clone());
        mem.add_tag(tag);
        assert_eq!(mem.tags.len(), 1);
    }

    #[test]
    fn add_or_update_backlink_keeps_higher_score() {
        let target = MemoryId::new();
        let mut mem = Memory::new("T", "C").unwrap();
        use crate::BacklinkKind;
        let low = Backlink::new(target, 0.5, BacklinkKind::Semantic).unwrap();
        let high = Backlink::new(target, 0.9, BacklinkKind::Semantic).unwrap();
        mem.add_or_update_backlink(low);
        mem.add_or_update_backlink(high);
        assert_eq!(mem.backlink_count(), 1);
        assert!((mem.backlinks[0].score - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn set_backlinks_dedupes_targets() {
        let target = MemoryId::new();
        let mut mem = Memory::new("T", "C").unwrap();
        use crate::BacklinkKind;
        let a = Backlink::new(target, 0.3, BacklinkKind::Semantic).unwrap();
        let b = Backlink::new(target, 0.8, BacklinkKind::Semantic).unwrap();
        mem.set_backlinks(vec![a, b]);
        assert_eq!(mem.backlink_count(), 1);
        assert!((mem.backlinks[0].score - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn update_content_refreshes_timestamp() {
        let mut mem = Memory::new("T", "C").unwrap();
        let before = mem.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(5));
        mem.update_content("Nouveau").unwrap();
        assert!(mem.updated_at > before);
    }
}