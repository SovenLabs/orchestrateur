//! `orch onboard` — première installation.

use std::path::Path;

use anyhow::Result;
use clap::Args;

use crate::harness_ops::{cmd_onboard, OnboardOptions};
use crate::tui;

/// Options de la commande onboard.
#[derive(Debug, Clone, Args)]
pub struct OnboardArgs {
    /// Profil sécurité (`local_only`, `ai_assisted`, …).
    #[arg(long)]
    pub profile: Option<String>,
    /// Provider LLM primaire.
    #[arg(long)]
    pub llm: Option<String>,
    /// Profil local souverain (ollama + zéro egress cloud). Activé par défaut.
    #[arg(long, default_value_t = true)]
    pub local_only: bool,
    /// Installe la tâche planifiée daemon Windows après onboard.
    #[arg(long)]
    pub install_daemon: bool,
    /// Mode interactif (questions guidées).
    #[arg(long)]
    pub interactive: bool,
}

/// Lance l'onboard (interactif ou via flags).
pub fn run(args: OnboardArgs, workspace: &Path) -> Result<()> {
    let use_wizard = args.interactive
        || (args.profile.is_none()
            && args.llm.is_none()
            && !args.local_only
            && !args.install_daemon);

    if use_wizard {
        tui::run_onboard_wizard(workspace)
    } else {
        cmd_onboard(
            workspace,
            &OnboardOptions {
                profile: args.profile,
                llm: args.llm,
                local_only: args.local_only,
                install_daemon: args.install_daemon,
            },
        )
    }
}