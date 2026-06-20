//! Tests de charge Cortex — ignorés par défaut.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Instant;

use cortex::{Backlink, BacklinkKind, KnowledgeGraph, Memory};

#[test]
#[ignore = "charge: 5000 mémoires, reconstruction KnowledgeGraph < 2s"]
fn perf_from_memories_5k() {
    let hub = Memory::new("hub", "contenu").unwrap();
    let mut memories = Vec::with_capacity(5000);
    memories.push(hub.clone());
    for i in 0..4999 {
        let mut mem = Memory::new(format!("M{i}"), "contenu").unwrap();
        if i % 3 == 0 {
            mem.set_backlinks(vec![Backlink::new(hub.id, 0.5, BacklinkKind::Semantic).unwrap()]);
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