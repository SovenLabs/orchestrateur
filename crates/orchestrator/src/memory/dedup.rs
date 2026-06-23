use std::collections::HashSet;

use cortex::MemoryDraft;

/// Score Jaccard entre deux ensembles de tokens (0.0–1.0).
#[must_use]
pub fn dedup_jaccard_score(a: &HashSet<String>, b: &HashSet<String>) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let inter = a.intersection(b).count() as f32;
    let union = a.union(b).count() as f32;
    inter / union
}

/// Tokenise titre + fichiers sources pour comparaison dédup.
#[must_use]
pub fn draft_token_set(draft: &MemoryDraft) -> HashSet<String> {
    let mut tokens = tokenize(&draft.title);
    for path in &draft.source_files {
        tokens.extend(tokenize(path));
    }
    tokens
}

fn tokenize(text: &str) -> HashSet<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '/' && c != '.' && c != '_')
        .filter(|t| t.len() >= 2)
        .map(str::to_string)
        .collect()
}

/// Indique si le brouillon est un doublon probable d'un existant (pré-LLM ou post-LLM).
#[must_use]
pub fn is_duplicate_draft(
    candidate: &MemoryDraft,
    existing: &[MemoryDraft],
    threshold: f32,
) -> bool {
    let cand = draft_token_set(candidate);
    existing
        .iter()
        .any(|other| dedup_jaccard_score(&cand, &draft_token_set(other)) >= threshold)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex::MemoryDraft;

    #[test]
    fn jaccard_identical_sets() {
        let a: HashSet<_> = ["rust", "api"].into_iter().map(str::to_string).collect();
        assert!((dedup_jaccard_score(&a, &a) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn detects_similar_drafts() {
        let a = MemoryDraft::new("Architecture API Rust", "body");
        let mut b = MemoryDraft::new("Architecture API Rust v2", "other");
        b.source_files = vec!["src/api.rs".into()];
        let mut a2 = MemoryDraft::new("Architecture API Rust", "body");
        a2.source_files = vec!["src/api.rs".into()];
        assert!(is_duplicate_draft(&a2, std::slice::from_ref(&b), 0.5));
        assert!(!is_duplicate_draft(&a, std::slice::from_ref(&b), 0.9));
    }
}