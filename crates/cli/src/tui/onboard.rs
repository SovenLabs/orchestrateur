//! Wizard `orchestrateur onboard` (premier lancement).

use std::path::Path;

use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Select};

use crate::harness_ops::{cmd_onboard, OnboardOptions};
use crate::tui::actions::block_on_action;
use crate::tui::bootstrap::wait_for_harness;
use crate::tui::menus::HarnessAction;
use crate::tui::progress::print_setup_card;
use crate::tui::theme::{self, format_menu_line, pause_enter};

struct WizardChoice {
    label: &'static str,
    hint: &'static str,
}

const PROFILE_CHOICES: &[WizardChoice] = &[
    WizardChoice {
        label: "local_only",
        hint: "zéro egress cloud, ollama local",
    },
    WizardChoice {
        label: "ai_assisted",
        hint: "LLM cloud autorisé selon profil",
    },
    WizardChoice {
        label: "strict",
        hint: "restrictions renforcées",
    },
    WizardChoice {
        label: "default",
        hint: "profil équilibré",
    },
];

const LLM_CHOICES: &[WizardChoice] = &[
    WizardChoice {
        label: "ollama",
        hint: "modèle local via Ollama",
    },
    WizardChoice {
        label: "xai",
        hint: "API xAI (clé XAI_API_KEY)",
    },
];

fn select_with_hints(title: &str, choices: &[WizardChoice]) -> Result<usize> {
    let items: Vec<String> = choices
        .iter()
        .map(|c| format_menu_line(c.label, c.hint, None))
        .collect();
    let theme = ColorfulTheme::default();
    Select::with_theme(&theme)
        .with_prompt(title)
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| anyhow::anyhow!(e))
}

/// Assistant interactif premier lancement.
pub fn run_onboard_wizard(workspace: &Path) -> Result<()> {
    theme::print_banner("onboard — Premier lancement", workspace);

    print_setup_card(
        "Configurons votre harness Orchestrateur",
        "Connectez un provider LLM pour commencer. La plupart des options prennent un clic.",
    );

    let config_path = workspace.join("config").join("orchestrator.toml");
    if config_path.exists() {
        println!("Config existante détectée : {}", config_path.display());
    } else {
        println!("Aucune config — création du workspace harness.");
    }
    println!();

    let profile_idx = select_with_hints("Profil sécurité", PROFILE_CHOICES)?;
    let profile = PROFILE_CHOICES[profile_idx].label;

    let llm = if profile == "local_only" {
        "ollama".to_string()
    } else {
        let llm_idx = select_with_hints("Provider LLM", LLM_CHOICES)?;
        LLM_CHOICES[llm_idx].label.to_string()
    };

    let install_items = [
        format_menu_line("oui", "tâche Windows au démarrage session", None),
        format_menu_line("non", "démarrage manuel du daemon", None),
    ];
    let install_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Installer le daemon automatiquement ?")
        .items(&install_items)
        .default(1)
        .interact()?;

    cmd_onboard(
        workspace,
        &OnboardOptions {
            profile: Some(profile.to_string()),
            llm: Some(llm),
            local_only: profile == "local_only",
            install_daemon: install_idx == 0,
        },
    )?;

    println!();
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(wait_for_harness(workspace))
    })?;

    println!();
    println!("Diagnostic harness…");
    block_on_action(workspace, HarnessAction::Doctor)?;

    println!();
    println!("Prochaine étape : orchestrateur setup → Gateway & canaux → Configurer Discord/Telegram");

    pause_enter("Onboard terminé. Entrée pour fermer.")?;
    Ok(())
}