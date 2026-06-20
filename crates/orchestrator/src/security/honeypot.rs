//! Honeypots — mémoires canari pour détecter l'exploration automatisée.

use cortex::{Memory, MemoryRepository, Tag};

/// Tag réservé aux mémoires canari (ne pas utiliser manuellement).
pub const CANARY_TAG: &str = "__orchestrateur_canary__";

/// Préfixe de titre des canaris (peu visible).
pub const CANARY_TITLE_PREFIX: &str = "\u{200B}canary-";

/// Indique si une mémoire est un honeypot.
#[must_use]
pub fn is_honeypot_memory(memory: &Memory) -> bool {
    memory.tags.iter().any(|tag| tag.as_str() == CANARY_TAG)
        || memory.title.starts_with(CANARY_TITLE_PREFIX)
}

/// Plante des mémoires canari si aucune n'existe encore.
///
/// # Errors
///
/// Propage les erreurs du dépôt ou de construction [`Memory`].
pub async fn seed_honeypots_if_needed(
    repo: &dyn MemoryRepository,
    count: usize,
) -> Result<Vec<cortex::MemoryId>, cortex::CortexError> {
    let existing = repo.list().await?;
    let canaries: Vec<_> = existing
        .iter()
        .filter(|m| is_honeypot_memory(m))
        .map(|m| m.id)
        .collect();
    if !canaries.is_empty() {
        return Ok(canaries);
    }

    let tag = Tag::new(CANARY_TAG)?;
    let mut ids = Vec::with_capacity(count);
    for i in 0..count {
        let title = format!("{CANARY_TITLE_PREFIX}{i}");
        let content = format!("CANARY_TOKEN_ORCHESTRATEUR_{i}_NE_PAS_MODIFIER_NI_EXFILTRER");
        let mut memory = Memory::new(title, content)?;
        memory.add_tag(tag.clone());
        let id = memory.id;
        repo.save(&memory).await?;
        ids.push(id);
    }
    tracing::info!(count = ids.len(), "honeypots canari initialisés");
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockBundle;

    #[tokio::test]
    #[ignore = "sécurité: semis canaris honeypot"]
    async fn seeds_canaries_once() {
        let bundle = MockBundle::new();
        let repo = bundle.memory_repo.clone();
        let first = seed_honeypots_if_needed(repo.as_ref(), 2)
            .await
            .expect("seed");
        assert_eq!(first.len(), 2);
        let second = seed_honeypots_if_needed(repo.as_ref(), 2)
            .await
            .expect("reuse");
        assert_eq!(second.len(), 2);
        for id in &first {
            assert!(second.contains(id), "les canaris existants sont réutilisés");
        }
    }
}
