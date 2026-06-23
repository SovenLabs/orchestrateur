//! Skills — trait, registre, hub et plugins dynamiques (Phase 11).

mod assimilate;
mod hub;
mod list_memories;
mod manifest;
mod marketplace;
#[cfg(feature = "plugins-native")]
mod native;
mod plugin;
mod registry;
mod search;
mod skill;

pub use assimilate::AssimilateSkill;
pub use hub::{HubError, SkillHubDescriptor, SkillsHub};
pub use marketplace::{
    best_skill_match, IntegrityReport, MarketplaceCatalog, MarketplaceEntry, MarketplaceError,
    MarketplaceSyncResult, SkillsMarketplace, suggest_skills,
};
pub use list_memories::ListMemoriesSkill;
pub use manifest::{
    compute_integrity_hash, load_manifest, verify_integrity_hash, ManifestError,
    NativePluginConfig, SkillManifest, SkillPluginConfig, SkillPluginKind, SubprocessPluginConfig,
};
#[cfg(feature = "plugins-native")]
pub use native::{NativePluginError, NativePluginSkill};
pub use plugin::SubprocessPluginSkill;
pub use registry::SkillRegistry;
pub use search::SearchMemoriesSkill;
pub use skill::{NoopSkill, Skill, SkillContext, SkillEntry, SkillOutput, SkillSource};

#[cfg(test)]
mod workspace_fixture;
