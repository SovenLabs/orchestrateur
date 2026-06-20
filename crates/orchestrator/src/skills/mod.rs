//! Squelette Skills — trait, registre et skill de démonstration.

mod registry;
mod skill;

pub use registry::SkillRegistry;
pub use skill::{NoopSkill, Skill, SkillContext, SkillOutput};