//! Types de skills spécialisées (Phase 6).

mod agent_skill;
mod b212_skill;
mod communication_skill;
mod cortex_skill;

pub use agent_skill::{AgentSkill, AgentSkillAdapter};
pub use b212_skill::{B212Skill, B212SkillAdapter};
pub use communication_skill::{CommunicationSkill, CommunicationSkillAdapter};
pub use cortex_skill::{CortexSkill, CortexSkillAdapter};