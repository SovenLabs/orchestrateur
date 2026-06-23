//! Modules de commandes CLI (verbe + objet).

pub mod config;
pub mod daemon;
pub mod doctor;
pub mod health;
pub mod memory;
pub mod onboard;
pub mod session;
pub mod skill;
pub mod uninstall;
pub mod update;

pub use config::ConfigCommands;
pub use daemon::DaemonCommands;
pub use memory::MemoryCommands;
pub use onboard::OnboardArgs;
pub use session::SessionCommands;
pub use skill::SkillCommands;
pub use update::UpdateArgs;