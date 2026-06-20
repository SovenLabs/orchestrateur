//! Use cases applicatifs — chaque flux est testable via mocks in-memory.

mod assimilate_from_draft;
mod get_memory;
mod list_memories;
mod save_memory;
mod search_memories;

pub use assimilate_from_draft::{AssimilateFromDraft, AssimilationResult};
pub use get_memory::GetMemory;
pub use list_memories::ListMemories;
pub use save_memory::SaveMemory;
pub use search_memories::SearchMemories;