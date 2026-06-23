//! Gestion marketplace et installation locale des skills (Phase 6).

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

mod installer;
mod manifest;
mod registry;
mod updater;

pub use installer::{InstallError, InstallResult, SkillInstaller};
pub use manifest::{SkillManifestTemplate, SkillTemplateKind};
pub use registry::{InstalledSkill, InstalledSkillRegistry, RegistryError};
pub use updater::{SkillUpdater, UpdateReport, UpdaterError};