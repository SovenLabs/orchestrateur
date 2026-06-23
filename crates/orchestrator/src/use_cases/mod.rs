//! Use cases applicatifs — chaque flux est testable via mocks in-memory.

mod assimilate_from_draft;
mod assimilate_from_text;
mod generate_insight_draft;
mod get_memory;
mod import_memories;
mod list_memories;
mod save_memory;
mod search_memories;

pub use assimilate_from_draft::{AssimilateFromDraft, AssimilationResult};
pub use assimilate_from_text::{AssimilateFromText, DEFAULT_ASSIMILATION_SYSTEM_PROMPT};
pub use generate_insight_draft::GenerateInsightDraft;
pub use get_memory::GetMemory;
pub use import_memories::{ImportMemories, ImportResult};
pub use list_memories::ListMemories;
pub use save_memory::SaveMemory;
pub use search_memories::SearchMemories;
