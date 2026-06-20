use crate::domain::{Backlink, BacklinkKind, CortexError, Memory, MemoryId};

/// Seuils configurables pour la génération de backlinks sémantiques.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimilarityThresholds {
    /// Score minimum pour créer un backlink sémantique.
    pub semantic_min: f32,
    /// Nombre maximum de backlinks par mémoire.
    pub max_links: usize,
}

impl Default for SimilarityThresholds {
    fn default() -> Self {
        Self {
            semantic_min: 0.75,
            max_links: 10,
        }
    }
}

/// Candidat à la création d'un backlink.
#[derive(Debug, Clone, PartialEq)]
pub struct BacklinkCandidate {
    /// Mémoire source du lien candidat.
    pub source: MemoryId,
    /// Mémoire cible du lien candidat.
    pub target: MemoryId,
    /// Score de similarité calculé.
    pub score: f32,
}

/// Service de domaine pur : calcule les backlinks par similarité cosinus.
///
/// Aucun I/O, aucun appel réseau — testable isolément.
pub struct BacklinkCalculator;

impl BacklinkCalculator {
    /// Calcule les backlinks sémantiques d'une mémoire vers un corpus.
    ///
    /// `embedding` : vecteur de la mémoire source.
    /// `corpus` : paires (id, embedding) des autres mémoires.
    pub fn compute_semantic_backlinks(
        source_id: MemoryId,
        embedding: &[f32],
        corpus: &[(MemoryId, Vec<f32>)],
        thresholds: SimilarityThresholds,
    ) -> Result<Vec<Backlink>, CortexError> {
        let mut candidates: Vec<BacklinkCandidate> = corpus
            .iter()
            .filter(|(id, _)| *id != source_id)
            .filter_map(|(id, vec)| {
                let score = cosine_similarity(embedding, vec)?;
                if score >= thresholds.semantic_min {
                    Some(BacklinkCandidate {
                        source: source_id,
                        target: *id,
                        score,
                    })
                } else {
                    None
                }
            })
            .collect();

        candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.truncate(thresholds.max_links);

        candidates
            .into_iter()
            .map(|c| Backlink::new(c.target, c.score, BacklinkKind::Semantic))
            .collect()
    }

    /// Extrait les wikilinks explicites `[[uuid]]` du contenu Markdown.
    pub fn extract_wikilinks(content: &str) -> Vec<MemoryId> {
        let mut ids = Vec::new();
        let mut rest = content;
        while let Some(start) = rest.find("[[") {
            rest = &rest[start + 2..];
            if let Some(end) = rest.find("]]") {
                let inner = rest[..end].trim();
                if let Ok(id) = inner.parse::<MemoryId>() {
                    ids.push(id);
                }
                rest = &rest[end + 2..];
            } else {
                break;
            }
        }
        ids
    }

    /// Fusionne backlinks sémantiques et wikilinks explicites (dédupliqués).
    pub fn merge_backlinks(
        semantic: Vec<Backlink>,
        explicit_targets: Vec<MemoryId>,
    ) -> Result<Vec<Backlink>, CortexError> {
        let mut merged = semantic;
        for target in explicit_targets {
            if merged.iter().any(|bl| bl.target == target) {
                continue;
            }
            merged.push(Backlink::new(target, 1.0, BacklinkKind::ExplicitWikilink)?);
        }
        Ok(merged)
    }

    /// Applique les backlinks calculés à une mémoire (mutation domaine).
    pub fn apply_to_memory(memory: &mut Memory, backlinks: Vec<Backlink>) {
        memory.set_backlinks(backlinks);
    }
}

/// Similarité cosinus entre deux vecteurs de même dimension.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.len() != b.len() || a.is_empty() {
        return None;
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return None;
    }
    Some((dot / (norm_a * norm_b)).clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_vec(x: f32, y: f32) -> Vec<f32> {
        vec![x, y]
    }

    #[test]
    fn cosine_identical_vectors() {
        let v = unit_vec(1.0, 0.0);
        let score = cosine_similarity(&v, &v).unwrap();
        assert!((score - 1.0).abs() < 1e-5);
    }

    #[test]
    fn cosine_orthogonal_vectors() {
        let score = cosine_similarity(&unit_vec(1.0, 0.0), &unit_vec(0.0, 1.0)).unwrap();
        assert!(score.abs() < 1e-5);
    }

    #[test]
    fn compute_respects_threshold_and_max() {
        let source = MemoryId::new();
        let e1 = (MemoryId::new(), unit_vec(1.0, 0.0));
        let e2 = (MemoryId::new(), unit_vec(0.99, 0.01));
        let e3 = (MemoryId::new(), unit_vec(0.0, 1.0));

        let thresholds = SimilarityThresholds {
            semantic_min: 0.5,
            max_links: 1,
        };

        let links = BacklinkCalculator::compute_semantic_backlinks(
            source,
            &unit_vec(1.0, 0.0),
            &[e1, e2, e3],
            thresholds,
        )
        .unwrap();

        assert_eq!(links.len(), 1);
        assert!(links[0].score >= 0.5);
    }

    #[test]
    fn extracts_wikilinks() {
        let id = MemoryId::new();
        let content = format!("Voir aussi [[{id}]] pour plus de détails.");
        let ids = BacklinkCalculator::extract_wikilinks(&content);
        assert_eq!(ids, vec![id]);
    }

    #[test]
    fn merge_deduplicates_explicit() {
        let target = MemoryId::new();
        let semantic = vec![Backlink::new(target, 0.8, BacklinkKind::Semantic).unwrap()];
        let merged =
            BacklinkCalculator::merge_backlinks(semantic.clone(), vec![target]).unwrap();
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].kind, BacklinkKind::Semantic);
    }

    #[test]
    fn apply_updates_memory() {
        let mut mem = Memory::new("T", "C").unwrap();
        let bl = Backlink::new(MemoryId::new(), 0.9, BacklinkKind::Semantic).unwrap();
        BacklinkCalculator::apply_to_memory(&mut mem, vec![bl.clone()]);
        assert_eq!(mem.backlinks, vec![bl]);
    }
}