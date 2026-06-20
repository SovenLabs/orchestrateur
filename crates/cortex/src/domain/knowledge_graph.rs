use std::collections::{HashMap, HashSet};

use super::memory::dedupe_backlinks;
use super::{Backlink, CortexError, Memory, MemoryId};

/// Graphe de connaissances reconstruit depuis les backlinks des mémoires.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct KnowledgeGraph {
    nodes: HashSet<MemoryId>,
    edges: HashMap<MemoryId, Vec<Backlink>>,
}

impl KnowledgeGraph {
    /// Crée un graphe vide.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reconstruit le graphe depuis une collection de mémoires.
    #[must_use]
    pub fn from_memories(memories: &[Memory]) -> Self {
        let mut graph = Self::new();
        for memory in memories {
            graph.insert_memory(memory);
        }
        graph
    }

    /// Insère ou met à jour un nœud et ses arêtes sortantes (idempotent).
    ///
    /// Les backlinks sont dédupliqués par cible (score maximal conservé).
    /// L'ensemble des nœuds (`HashSet`) évite les doublons sur réinsertion.
    pub fn insert_memory(&mut self, memory: &Memory) {
        self.nodes.insert(memory.id);
        let links = dedupe_backlinks(memory.backlinks.clone());
        self.edges.insert(memory.id, links);
        if let Some(links) = self.edges.get(&memory.id) {
            for backlink in links {
                self.nodes.insert(backlink.target);
            }
        }
    }

    /// Retourne les backlinks sortants d'une mémoire.
    #[must_use]
    pub fn outgoing(&self, id: MemoryId) -> &[Backlink] {
        self.edges.get(&id).map_or(&[][..], Vec::as_slice)
    }

    /// Voisins directs (cibles des backlinks).
    #[must_use]
    pub fn neighbors(&self, id: MemoryId) -> Vec<MemoryId> {
        self.outgoing(id)
            .iter()
            .map(|bl| bl.target)
            .collect()
    }

    /// Nombre de nœuds connus.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Nombre total d'arêtes.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.values().map(Vec::len).sum()
    }

    /// Vérifie l'intégrité : toutes les cibles existent comme nœuds.
    ///
    /// # Errors
    ///
    /// Retourne [`CortexError::GraphError`] si une arête pointe vers un nœud absent.
    pub fn validate(&self) -> Result<(), CortexError> {
        for (source, links) in &self.edges {
            for link in links {
                if !self.nodes.contains(&link.target) {
                    return Err(CortexError::GraphError(format!(
                        "arête {source} → {} pointe vers un nœud inconnu",
                        link.target
                    )));
                }
            }
        }
        Ok(())
    }

    /// Mémoires triées par nombre de backlinks entrants (hubs).
    #[must_use]
    pub fn hub_ranking(&self) -> Vec<(MemoryId, usize)> {
        let mut inbound: HashMap<MemoryId, usize> = HashMap::new();
        for links in self.edges.values() {
            for link in links {
                *inbound.entry(link.target).or_default() += 1;
            }
        }
        let mut ranking: Vec<_> = inbound.into_iter().collect();
        ranking.sort_by_key(|b| std::cmp::Reverse(b.1));
        ranking
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::BacklinkKind;

    fn sample_memory(title: &str, links: Vec<Backlink>) -> Memory {
        let mut mem = Memory::new(title, "contenu").unwrap();
        mem.set_backlinks(links);
        mem
    }

    #[test]
    fn builds_from_memories() {
        let a = Memory::new("A", "ca").unwrap();
        let b = sample_memory("B", vec![Backlink::new(a.id, 0.8, BacklinkKind::Semantic).unwrap()]);

        let graph = KnowledgeGraph::from_memories(&[a, b]);
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn hub_ranking_counts_inbound() {
        let hub = Memory::new("hub", "c").unwrap();
        let spoke = sample_memory(
            "spoke",
            vec![Backlink::new(hub.id, 0.7, BacklinkKind::ExplicitWikilink).unwrap()],
        );
        let graph = KnowledgeGraph::from_memories(&[hub.clone(), spoke]);
        let ranking = graph.hub_ranking();
        assert_eq!(ranking[0].0, hub.id);
        assert_eq!(ranking[0].1, 1);
    }

    #[test]
    fn insert_memory_is_idempotent() {
        let mut mem = Memory::new("A", "c").unwrap();
        let target = MemoryId::new();
        let bl = Backlink::new(target, 0.5, BacklinkKind::Semantic).unwrap();
        mem.add_or_update_backlink(bl);

        let mut graph = KnowledgeGraph::new();
        graph.insert_memory(&mem);
        graph.insert_memory(&mem);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn dedupes_duplicate_backlink_targets() {
        let target = MemoryId::new();
        let mut mem = Memory::new("A", "c").unwrap();
        let source = mem.id;
        mem.set_backlinks(vec![
            Backlink::new(target, 0.4, BacklinkKind::Semantic).unwrap(),
            Backlink::new(target, 0.9, BacklinkKind::Semantic).unwrap(),
        ]);

        let graph = KnowledgeGraph::from_memories(&[mem]);
        assert_eq!(graph.edge_count(), 1);
        assert!((graph.outgoing(source)[0].score - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn neighbors_returns_targets() {
        let target = MemoryId::new();
        let mem = sample_memory(
            "x",
            vec![Backlink::new(target, 1.0, BacklinkKind::Semantic).unwrap()],
        );
        let source_id = mem.id;
        let graph = KnowledgeGraph::from_memories(&[mem]);
        assert_eq!(graph.neighbors(source_id), vec![target]);
    }
}