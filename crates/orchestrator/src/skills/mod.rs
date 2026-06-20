//! Skills — trait, registre et capacités opérationnelles.

mod assimilate;
mod list_memories;
mod registry;
mod search;
mod skill;

pub use assimilate::AssimilateSkill;
pub use list_memories::ListMemoriesSkill;
pub use registry::SkillRegistry;
pub use search::SearchMemoriesSkill;
pub use skill::{NoopSkill, Skill, SkillContext, SkillOutput};
