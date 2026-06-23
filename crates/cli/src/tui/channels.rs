//! Configuration guidée des canaux messaging (style Hermes).

use std::path::Path;

use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};
use orchestrator::gateway::resolve_channel_config;
use orchestrator::OrchestratorConfig;

use crate::present::{channels_enable, channels_status, set_user_env_var};

use super::theme::{format_menu_line, print_banner, print_breadcrumb};

/// Descripteur UI d'un canal configurable.
struct ChannelGuide {
    id: &'static str,
    display_name: &'static str,
    token_env: &'static str,
    token_label: &'static str,
    setup_url: &'static str,
    setup_blurb: &'static str,
    allowed_env: Option<&'static str>,
    allowed_label: Option<&'static str>,
}

const CONFIGURABLE: &[ChannelGuide] = &[
    ChannelGuide {
        id: "telegram",
        display_name: "Telegram",
        token_env: "TELEGRAM_BOT_TOKEN",
        token_label: "Token bot Telegram",
        setup_url: "https://core.telegram.org/bots/tutorial",
        setup_blurb: "Ouvrez @BotFather sur Telegram, créez un bot, copiez le token.",
        allowed_env: Some("ORCHESTRATEUR_TELEGRAM_ALLOWED_USER_IDS"),
        allowed_label: Some("IDs utilisateur Telegram autorisés (recommandé)"),
    },
    ChannelGuide {
        id: "discord",
        display_name: "Discord",
        token_env: "DISCORD_BOT_TOKEN",
        token_label: "Token bot Discord",
        setup_url: "https://discord.com/developers/applications",
        setup_blurb: "Developer Portal → Application → Bot → copiez le token. Invitez le bot sur votre serveur.",
        allowed_env: Some("ORCHESTRATEUR_DISCORD_ALLOWED_USER_IDS"),
        allowed_label: Some("IDs utilisateur Discord autorisés (recommandé)"),
    },
    ChannelGuide {
        id: "slack",
        display_name: "Slack",
        token_env: "SLACK_BOT_TOKEN",
        token_label: "Token bot Slack",
        setup_url: "https://api.slack.com/apps",
        setup_blurb: "Créez une app Slack, ajoutez les scopes bot, installez sur le workspace.",
        allowed_env: None,
        allowed_label: None,
    },
];

fn find_guide(id: &str) -> Option<&'static ChannelGuide> {
    CONFIGURABLE.iter().find(|g| g.id == id)
}

/// Badges d'état pour un canal (`[actif]`, `[needs setup]`, …).
pub fn channel_badges(workspace: &Path, channel_id: &str) -> Vec<String> {
    let Ok(config) = OrchestratorConfig::load_workspace(workspace) else {
        return vec!["[config absente]".into()];
    };
    let cfg = resolve_channel_config(&config.gateway, channel_id);
    let mut badges = Vec::new();
    if cfg.enabled {
        badges.push("[actif]".into());
    } else {
        badges.push("[désactivé]".into());
    }
    if !cfg.token_env.is_empty() && std::env::var(&cfg.token_env).is_err() {
        badges.push("[needs setup]".into());
    }
    badges
}

fn print_channel_header(workspace: &Path, guide: &ChannelGuide) {
    let badges = channel_badges(workspace, guide.id);
    let badge_str = badges.join(" ");
    println!();
    println!("{}  {}", guide.display_name, badge_str);
    println!("{}", guide.setup_blurb);
    println!();
    println!("OBTENIR VOS IDENTIFIANTS");
    println!("  Guide : {}", guide.setup_url);
    println!();
}

/// Assistant de configuration pour un canal donné.
pub fn run_channel_configure(workspace: &Path, channel_id: &str) -> Result<()> {
    let guide = find_guide(channel_id)
        .with_context(|| format!("canal non configurable via assistant : {channel_id}"))?;

    print_banner(&format!("Configurer — {}", guide.display_name), workspace);
    print_breadcrumb(&["gateway", "canaux", guide.id]);
    print_channel_header(workspace, guide);

    let has_token = std::env::var(guide.token_env).is_ok();
    if has_token {
        println!(
            "Token déjà défini ({}) — laissez vide pour conserver.",
            guide.token_env
        );
    }

    let token: String = Password::new()
        .with_prompt(guide.token_label)
        .allow_empty_password(true)
        .interact()
        .map_err(|e| anyhow::anyhow!(e))?;

    if !token.trim().is_empty() {
        set_user_env_var(guide.token_env, token.trim())?;
        println!("Variable utilisateur enregistrée : {}", guide.token_env);
    } else if !has_token {
        println!("⚠ Token absent — le canal restera en [needs setup].");
    }

    if let (Some(env), Some(label)) = (guide.allowed_env, guide.allowed_label) {
        println!();
        println!("RECOMMANDÉ");
        let current = std::env::var(env).unwrap_or_default();
        let allowed: String = Input::new()
            .with_prompt(label)
            .default(current)
            .allow_empty(true)
            .interact_text()
            .map_err(|e| anyhow::anyhow!(e))?;
        if !allowed.trim().is_empty() {
            set_user_env_var(env, allowed.trim())?;
            println!("Enregistré : {env}");
        }
    }

    println!();
    let enable = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Activer {} dans orchestrator.toml ?", guide.display_name))
        .default(true)
        .interact()
        .map_err(|e| anyhow::anyhow!(e))?;

    if enable {
        channels_enable(workspace, guide.id)?;
    }

    println!();
    channels_status(workspace)?;
    Ok(())
}

/// Sélecteur de canal puis configuration guidée.
pub fn run_channel_wizard(workspace: &Path) -> Result<()> {
    print_banner("Canaux messaging — configuration", workspace);
    print_breadcrumb(&["gateway", "canaux"]);

    let labels: Vec<String> = CONFIGURABLE
        .iter()
        .map(|g| {
            let badges = channel_badges(workspace, g.id).join(" ");
            format_menu_line(g.display_name, "guide credentials + token", Some(&badges))
        })
        .collect();

    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choisir un canal à configurer")
        .items(&labels)
        .default(0)
        .interact()
        .map_err(|e| anyhow::anyhow!(e))?;

    run_channel_configure(workspace, CONFIGURABLE[idx].id)
}

/// Libellés dynamiques pour le sous-menu canaux (badges live).
pub fn gateway_channel_menu_labels(workspace: &Path) -> Vec<String> {
    let mut lines = vec![
        format_menu_line("← Retour", "menu gateway", None),
        format_menu_line(
            "Configurer un canal",
            "assistant credentials (Telegram, Discord, Slack)",
            None,
        ),
        format_menu_line("Lister les canaux", "catalogue enregistré", None),
        format_menu_line("Statut canaux", "enabled + variables token", None),
    ];

    for guide in CONFIGURABLE {
        let badges = channel_badges(workspace, guide.id).join(" ");
        lines.push(format_menu_line(
            &format!("Configurer {}", guide.display_name),
            "token + activation",
            Some(&badges),
        ));
    }

    lines
}

/// Action correspondant à un index du menu canaux dynamique.
pub fn gateway_channel_action(index: usize) -> Option<ChannelMenuAction> {
    match index {
        0 => Some(ChannelMenuAction::Back),
        1 => Some(ChannelMenuAction::Wizard),
        2 => Some(ChannelMenuAction::List),
        3 => Some(ChannelMenuAction::Status),
        i if i >= 4 && i < 4 + CONFIGURABLE.len() => {
            Some(ChannelMenuAction::Configure(CONFIGURABLE[i - 4].id))
        }
        _ => None,
    }
}

/// Action du menu canaux dynamique.
#[derive(Debug, Clone, Copy)]
pub enum ChannelMenuAction {
    Back,
    Wizard,
    List,
    Status,
    Configure(&'static str),
}