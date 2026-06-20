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
}