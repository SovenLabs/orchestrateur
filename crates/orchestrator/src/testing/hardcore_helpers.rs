use cortex::{DomainEvent, KnowledgeGraph, Memory, MemoryId};

use crate::deps::AppDependencies;
use crate::OrchestratorFacade;

/// Calcule un percentile sur des échantillons de latence (millisecondes).
#[must_use]
pub fn percentile_ms(mut samples: Vec<u128>, percentile: u8) -> u128 {
    if samples.is_empty() {
        return 0;
    }
    samples.sort_unstable();
    let last = samples.len() - 1;
    let pct = usize::from(percentile.min(100));
    let idx = (last * pct / 100).min(last);
    samples[idx]
}

/// Construit une facade de test à partir de dépendances complètes.
#[must_use]
pub fn build_test_facade(deps: AppDependencies) -> OrchestratorFacade {
    OrchestratorFacade::new(deps)
}

/// Vérifie la cohérence graphe + vector store + dépôt fichiers.
///
/// # Errors
///
/// Retourne une description lisible si une incohérence est détectée.
pub async fn assert_workspace_consistent(deps: &AppDependencies) -> Result<(), String> {
    let memories = deps
        .memory_repo
        .list()
        .await
        .map_err(|e| e.to_string())?;

    let graph = KnowledgeGraph::from_memories(&memories);
    graph.validate().map_err(|e| e.to_string())?;
    graph.validate_resolvable(&memories)
        .map_err(|e| e.to_string())?;

    for mem in &memories {
        let emb = deps
            .vector_store
            .get_embedding(mem.id)
            .await
            .map_err(|e| e.to_string())?;
        if emb.is_none() {
            return Err(format!(
                "embedding absent pour la mémoire {} ({})",
                mem.id, mem.title
            ));
        }
    }

    Ok(())
}

/// Crée une mémoire de test indexée par un entier.
///
/// # Errors
///
/// Propage [`cortex::CortexError`] si la création du domaine échoue.
pub fn test_memory(index: usize) -> Result<Memory, cortex::CortexError> {
    Memory::new(
        format!("Mémoire hardcore #{index}"),
        format!("Contenu sémantique numéro {index} pour tests de charge."),
    )
}

/// Crée N mémoires distinctes.
///
/// # Errors
///
/// Propage [`cortex::CortexError`] si une création échoue.
pub fn test_memories(count: usize) -> Result<Vec<Memory>, cortex::CortexError> {
    (0..count).map(test_memory).collect()
}

/// Vérifie qu'aucun ID du graphe ne pointe vers une mémoire fantôme.
///
/// # Errors
///
/// Retourne une description si un backlink cible une mémoire absente.
pub async fn assert_no_ghost_nodes(deps: &AppDependencies) -> Result<(), String> {
    let memories = deps
        .memory_repo
        .list()
        .await
        .map_err(|e| e.to_string())?;
    let ids: std::collections::HashSet<MemoryId> = memories.iter().map(|m| m.id).collect();

    for mem in &memories {
        for bl in &mem.backlinks {
            if !ids.contains(&bl.target) {
                return Err(format!(
                    "backlink fantôme {} -> {} dans {}",
                    mem.id, bl.target, mem.title
                ));
            }
        }
    }
    Ok(())
}

/// Compte les événements d'un type donné via un publisher collecteur.
#[must_use]
pub fn count_domain_events(events: &[DomainEvent], variant: &str) -> usize {
    events
        .iter()
        .filter(|e| {
            matches!(
                (variant, e),
                ("assimilated", DomainEvent::MemoryAssimilated(_))
                    | ("graph", DomainEvent::KnowledgeGraphValidated(_))
            )
        })
        .count()
}