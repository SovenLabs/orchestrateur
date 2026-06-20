use std::collections::{HashMap, HashSet};

use super::{deduplicate, Backlink, CortexError, Memory, MemoryId};

/// Graphe de connaissances avec index sortant et entrant maintenus à l'insertion.
///
/// ## Coût de construction
///
/// [`KnowledgeGraph::from_memories`] parcourt toutes les mémoires une fois — **O(n × b)**
/// où *n* est le nombre de mémoires et *b* le nombre moyen de backlinks par mémoire.
/// Pour des corpus très volumineux (≥ 10 000 mémoires), préférer des insertions
/// incrémentales via [`insert_memory`] plutôt qu'une reconstruction complète répétée.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct KnowledgeGraph {
    nodes: HashSet<MemoryId>,
    /// Arêtes sortantes : source → backlinks.
    edges: HashMap<MemoryId, Vec<Backlink>>,
    /// Index entrant : cible → (source → backlink depuis source).
    inbound_index: HashMap<MemoryId, HashMap<MemoryId, Backlink>>,
    /// Cache matérialisé pour [`Self::incoming`].
    inbound_cache: HashMap<MemoryId, Vec<Backlink>>,
}

impl KnowledgeGraph {
    /// Crée un graphe vide.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reconstruit le graphe depuis une collection de mémoires.
    ///
    /// Complexité : **O(n × b)** — une passe par mémoire avec insertion incrémentale.
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
    /// L'index entrant est mis à jour de façon incrémentale — **O(b)** pour cette mémoire.
    pub fn insert_memory(&mut self, memory: &Memory) {
        if let Some(previous) = self.edges.remove(&memory.id) {
            for bl in &previous {
                self.remove_inbound_link(memory.id, bl.target);
            }
        }

        self.nodes.insert(memory.id);
        let links = deduplicate(memory.backlinks.clone());
        for bl in &links {
            self.nodes.insert(bl.target);
            self.add_inbound_link(memory.id, bl);
        }
        self.edges.insert(memory.id, links);
    }

    /// Retourne les backlinks sortants d'une mémoire.
    #[must_use]
    pub fn outgoing(&self, id: MemoryId) -> &[Backlink] {
        self.edges.get(&id).map_or(&[][..], Vec::as_slice)
    }

    /// Retourne les backlinks entrants vers une mémoire (toutes sources confondues).
    #[must_use]
    pub fn incoming(&self, id: MemoryId) -> &[Backlink] {
        self.inbound_cache.get(&id).map_or(&[][..], Vec::as_slice)
    }

    /// Nombre de sources pointant vers cette mémoire (inbound distinct par source).
    #[must_use]
    pub fn inbound_count(&self, id: MemoryId) -> usize {
        self.inbound_index
            .get(&id)
            .map_or(0, HashMap::len)
    }

    /// Voisins directs (cibles des backlinks sortants).
    #[must_use]
    pub fn neighbors(&self, id: MemoryId) -> Vec<MemoryId> {
        self.outgoing(id).iter().map(|bl| bl.target).collect()
    }

    /// Nombre de nœuds connus.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Nombre total d'arêtes sortantes.
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

    /// Vérifie que chaque cible de backlink correspond à une mémoire existante.
    ///
    /// # Errors
    ///
    /// Retourne [`CortexError::MemoryNotFound`] si une cible n'a pas de mémoire correspondante.
    pub fn validate_resolvable(&self, memories: &[Memory]) -> Result<(), CortexError> {
        let ids: HashSet<MemoryId> = memories.iter().map(|m| m.id).collect();
        for mem in memories {
            for bl in &mem.backlinks {
                if !ids.contains(&bl.target) {
                    return Err(CortexError::MemoryNotFound(bl.target));
                }
            }
        }
        Ok(())
    }

    /// Mémoires triées par nombre de backlinks entrants (hubs).
    ///
    /// Complexité : **O(h log h)** où *h* est le nombre de nœuds avec inbound > 0.
    #[must_use]
    pub fn hub_ranking(&self) -> Vec<(MemoryId, usize)> {
        let mut ranking: Vec<_> = self
            .inbound_index
            .iter()
            .filter_map(|(id, sources)| {
                let count = sources.len();
                if count > 0 {
                    Some((*id, count))
                } else {
                    None
                }
            })
            .collect();
        ranking.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        ranking
    }

    fn add_inbound_link(&mut self, source: MemoryId, backlink: &Backlink) {
        self.inbound_index
            .entry(backlink.target)
            .or_default()
            .insert(source, backlink.clone());
        self.refresh_inbound_cache(backlink.target);
    }

    fn remove_inbound_link(&mut self, source: MemoryId, target: MemoryId) {
        if let Some(map) = self.inbound_index.get_mut(&target) {
            map.remove(&source);
            if map.is_empty() {
                self.inbound_index.remove(&target);
            }
        }
        self.refresh_inbound_cache(target);
    }

    fn refresh_inbound_cache(&mut self, target: MemoryId) {
        match self.inbound_index.get(&target) {
            Some(map) if !map.is_empty() => {
                self.inbound_cache
                    .insert(target, map.values().cloned().collect());
            }
            _ => {
                self.inbound_cache.remove(&target);
            }
        }
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
        let b = sample_memory(
            "B",
            vec![Backlink::new(a.id, 0.8, BacklinkKind::Semantic).unwrap()],
        );

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
    fn incoming_returns_backlinks_to_target() {
        let hub = Memory::new("hub", "c").unwrap();
        let spoke = sample_memory(
            "spoke",
            vec![Backlink::new(hub.id, 0.7, BacklinkKind::Semantic).unwrap()],
        );
        let graph = KnowledgeGraph::from_memories(&[hub.clone(), spoke]);
        assert_eq!(graph.inbound_count(hub.id), 1);
        assert_eq!(graph.incoming(hub.id).len(), 1);
        assert_eq!(graph.incoming(hub.id)[0].target, hub.id);
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
        assert_eq!(graph.inbound_count(target), 1);
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
    fn validate_resolvable_rejects_missing_target() {
        let target = MemoryId::new();
        let mem = sample_memory(
            "x",
            vec![Backlink::new(target, 1.0, BacklinkKind::Semantic).unwrap()],
        );
        let graph = KnowledgeGraph::from_memories(std::slice::from_ref(&mem));
        assert!(graph
            .validate_resolvable(std::slice::from_ref(&mem))
            .is_err());
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

    #[test]
    fn insert_memory_updates_inbound_on_backlink_change() {
        let a = Memory::new("A", "c").unwrap();
        let b = Memory::new("B", "c").unwrap();
        let c = Memory::new("C", "c").unwrap();

        let mut graph = KnowledgeGraph::new();
        let mut mem = sample_memory("linker", vec![Backlink::new(a.id, 0.5, BacklinkKind::Semantic).unwrap()]);
        graph.insert_memory(&mem);

        assert_eq!(graph.inbound_count(a.id), 1);

        mem.set_backlinks(vec![Backlink::new(b.id, 0.6, BacklinkKind::Semantic).unwrap()]);
        graph.insert_memory(&mem);

        assert_eq!(graph.inbound_count(a.id), 0);
        assert_eq!(graph.inbound_count(b.id), 1);
        let _ = c;
    }

    #[test]
    #[ignore = "perf — 5000 mémoires, reconstruction graphe < 2s en debug"]
    fn perf_from_memories_5k() {
        use std::time::Instant;

        let hub = Memory::new("hub", "contenu").unwrap();
        let mut memories = Vec::with_capacity(5000);
        memories.push(hub.clone());
        for i in 0..4999 {
            let mut mem = Memory::new(format!("M{i}"), "contenu").unwrap();
            if i % 3 == 0 {
                mem.set_backlinks(vec![
                    Backlink::new(hub.id, 0.5, BacklinkKind::Semantic).unwrap(),
                ]);
            }
            memories.push(mem);
        }

        let start = Instant::now();
        let graph = KnowledgeGraph::from_memories(&memories);
        let elapsed = start.elapsed();

        assert_eq!(graph.node_count(), 5000);
        assert!(graph.hub_ranking()[0].0 == hub.id);
        assert!(
            elapsed.as_secs() < 2,
            "from_memories 5k trop lent: {elapsed:?}"
        );
    }
}