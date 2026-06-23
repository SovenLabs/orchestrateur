//! Tests property-based sur les invariants du domaine Cortex.

use cortex::{Memory, MemoryId, Tag};
use proptest::prelude::*;

proptest! {
    #[test]
    fn memory_new_trims_and_preserves_non_empty(title in "\\PC{1,20}", content in "\\PC{1,40}") {
        let padded_title = format!("  {title}  ");
        let padded_content = format!("  {content}  ");
        let mem = Memory::new(padded_title, padded_content).expect("mémoire valide");
        prop_assert_eq!(mem.title, title.trim());
        prop_assert_eq!(mem.content, content.trim());
    }

    #[test]
    fn memory_id_from_str_only_accepts_v7(bytes in any::<[u8; 16]>()) {
        let uuid = uuid::Uuid::from_bytes(bytes);
        let result = uuid.to_string().parse::<MemoryId>();
        if uuid.get_version() == Some(uuid::Version::SortRand) {
            prop_assert!(result.is_ok());
        } else {
            prop_assert!(result.is_err());
        }
    }

    #[test]
    fn tag_normalization_is_idempotent(raw in "[a-z0-9_-]{1,32}") {
        let tag = Tag::new(&raw).expect("tag valide");
        let again = Tag::new(tag.as_str()).expect("re-normalisation");
        prop_assert_eq!(tag, again);
    }

    #[test]
    fn knowledge_graph_node_count_matches_inserted_memories(
        count in 1usize..32,
        titles in prop::collection::vec("[a-zA-Z0-9][a-zA-Z0-9 _-]{0,11}", 1..32),
    ) {
        let n = count.min(titles.len()).max(1);
        let memories: Vec<Memory> = titles
            .into_iter()
            .take(n)
            .map(|t| Memory::new(t, "contenu test").expect("mémoire"))
            .collect();
        let graph = cortex::KnowledgeGraph::from_memories(&memories);
        prop_assert!(graph.node_count() >= n);
        prop_assert!(graph.validate().is_ok());
    }

    #[test]
    fn hub_ranking_inbound_never_exceeds_edge_sources(
        hub_title in "[a-zA-Z0-9][a-zA-Z0-9 _-]{0,9}",
        spoke_count in 1usize..8,
    ) {
        let hub = Memory::new(hub_title, "hub").expect("hub");
        let spokes: Vec<Memory> = (0..spoke_count)
            .map(|i| {
                let mut mem = Memory::new(format!("spoke-{i}"), "c").expect("spoke");
                mem.set_backlinks(vec![
                    cortex::Backlink::new(hub.id, 0.5, cortex::BacklinkKind::Semantic)
                        .expect("backlink"),
                ]);
                mem
            })
            .collect();
        let mut all = vec![hub.clone()];
        all.extend(spokes);
        let graph = cortex::KnowledgeGraph::from_memories(&all);
        let ranking = graph.hub_ranking();
        if let Some((id, inbound)) = ranking.first() {
            prop_assert_eq!(*id, hub.id);
            prop_assert_eq!(*inbound, spoke_count);
        }
    }
}