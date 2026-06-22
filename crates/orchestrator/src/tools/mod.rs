//! Registre d'outils agent Phase 7.

mod mcp_call;
mod mcp_list;
mod memory_assimilate;
mod memory_get;
mod memory_search;
mod registry;
mod skill_execute;
mod skill_list;
mod skill_suggest;
mod tool;
mod toolsets;

pub use toolsets::{ToolsetDescriptor, ToolsetRegistry, TOOLSET_DESCRIPTORS};

pub use memory_assimilate::MemoryAssimilateTool;
pub use memory_get::MemoryGetTool;
pub use memory_search::MemorySearchTool;
pub use skill_execute::SkillExecuteTool;
pub use skill_list::SkillListTool;
pub use skill_suggest::SkillSuggestTool;
pub use registry::ToolRegistry;
pub use tool::{Tool, ToolContext, ToolDefinition, ToolError, ToolResult};