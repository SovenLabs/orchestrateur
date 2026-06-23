//! Skills — trait, registre, hub, loader et plugins dynamiques (Phase 6).

mod assimilate;
mod context;
mod dependencies;
mod hub;
mod hot_reload;
mod list_memories;
mod loader;
mod manifest;
mod metadata;
mod marketplace;
#[cfg(feature = "plugins-native")]
mod native;
mod plugin;
mod registry;
mod search;
mod skill;
mod r#trait;
mod types;

pub use assimilate::AssimilateSkill;
pub use context::{SkillExecution, SkillHostContext};
pub use dependencies::{resolve_load_order, DependencyError};
pub use hub::{register_manifest, HubError, SkillHubDescriptor, SkillsHub};
pub use hot_reload::{SkillHotReload, HotReloadError};
pub use loader::{LoaderError, SkillLoader};
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
pub use metadata::{SkillMetadata, SkillType};
pub use r#trait::{Skill, TypedSkill};
pub use skill::{NoopSkill, SkillContext, SkillEntry, SkillOutput, SkillSource};
pub use types::{
    AgentSkill, AgentSkillAdapter, B212Skill, B212SkillAdapter, CommunicationSkill,
    CommunicationSkillAdapter, CortexSkill, CortexSkillAdapter,
};

#[cfg(test)]
mod workspace_fixture;