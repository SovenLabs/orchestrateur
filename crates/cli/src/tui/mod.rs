//! Menus interactifs harness (`setup`, `settings`, `onboard`).

mod actions;
mod bootstrap;
mod channels;
mod menus;
mod onboard;
mod progress;
mod theme;

use std::path::Path;

use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Select};

use menus::{MenuAction, SubMenuId};
use theme::{
    format_menu_line, pause_enter, print_banner, print_breadcrumb, print_footer, print_status_chips,
};

use crate::present::harness_service_badges;

pub use onboard::run_onboard_wizard;

/// Centre de commande harness (`orchestrateur setup`).
pub fn run_setup(workspace: &Path) -> Result<()> {
    run_menu(workspace, SubMenuId::SetupRoot, false)
}

/// Configuration harness (`orchestrateur settings`).
pub fn run_settings(workspace: &Path) -> Result<()> {
    run_menu(workspace, SubMenuId::SettingsRoot, false)
}

fn run_menu(workspace: &Path, root: SubMenuId, from_setup_settings_link: bool) -> Result<()> {
    let mut stack = vec![root];

    loop {
        let current = *stack.last().expect("stack menu");

        if current == SubMenuId::GatewayChannels {
            run_gateway_channels_menu(workspace, &mut stack)?;
            continue;
        }

        let title = menus::title_for(current);
        print_banner(title, workspace);

        let crumbs: Vec<&str> = stack.iter().map(|id| menu_short_title(*id)).collect();
        print_breadcrumb(&crumbs);

        if matches!(current, SubMenuId::Daemon | SubMenuId::Gateway | SubMenuId::Health) {
            let (daemon, gateway) = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(harness_service_badges(workspace))
            });
            print_status_chips(&[
                ("daemon", daemon.badge()),
                ("gateway", gateway.badge()),
            ]);
        }

        let items = menus::items_for(current);
        let labels: Vec<String> = items
            .iter()
            .map(|item| format_menu_line(item.label, item.hint, None))
            .collect();

        print_footer();
        let theme = ColorfulTheme::default();
        let selection = Select::with_theme(&theme)
            .with_prompt("Choisir une action")
            .items(&labels)
            .default(0)
            .interact_opt()
            .map_err(|e| anyhow::anyhow!(e))?;

        let Some(index) = selection else {
            if stack.len() > 1 {
                stack.pop();
            } else {
                return Ok(());
            }
            continue;
        };

        let item = items[index];
        match item.action {
            MenuAction::Back => {
                if stack.len() > 1 {
                    stack.pop();
                } else if from_setup_settings_link && current == SubMenuId::SettingsRoot {
                    return Ok(());
                }
            }
            MenuAction::Quit => return Ok(()),
            MenuAction::OpenSettings => {
                run_menu(workspace, SubMenuId::SettingsRoot, true)?;
            }
            MenuAction::SubMenu(id) => stack.push(id),
            MenuAction::Run(action) => {
                if let Err(err) = actions::block_on_action(workspace, action) {
                    eprintln!("\nErreur : {err:#}");
                }
                let _ = pause_enter("Entrée pour revenir au menu…");
            }
        }
    }
}

fn run_gateway_channels_menu(workspace: &Path, stack: &mut Vec<SubMenuId>) -> Result<()> {
    loop {
        print_banner(menus::title_for(SubMenuId::GatewayChannels), workspace);
        print_breadcrumb(&["gateway", "canaux"]);

        let (_, gateway) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(harness_service_badges(workspace))
        });
        print_status_chips(&[("gateway", gateway.badge())]);

        let labels = channels::gateway_channel_menu_labels(workspace);
        print_footer();
        let theme = ColorfulTheme::default();
        let selection = Select::with_theme(&theme)
            .with_prompt("Choisir une action")
            .items(&labels)
            .default(1)
            .interact_opt()
            .map_err(|e| anyhow::anyhow!(e))?;

        let Some(index) = selection else {
            if stack.len() > 1 {
                stack.pop();
            }
            return Ok(());
        };

        match channels::gateway_channel_action(index) {
            Some(channels::ChannelMenuAction::Back) => {
                if stack.len() > 1 {
                    stack.pop();
                }
                return Ok(());
            }
            Some(channels::ChannelMenuAction::Wizard) => {
                if let Err(err) = channels::run_channel_wizard(workspace) {
                    eprintln!("\nErreur : {err:#}");
                }
                let _ = pause_enter("Entrée pour revenir au menu…");
            }
            Some(channels::ChannelMenuAction::List) => {
                if let Err(err) = actions::block_on_action(workspace, menus::HarnessAction::ChannelsList) {
                    eprintln!("\nErreur : {err:#}");
                }
                let _ = pause_enter("Entrée pour revenir au menu…");
            }
            Some(channels::ChannelMenuAction::Status) => {
                if let Err(err) = actions::block_on_action(workspace, menus::HarnessAction::ChannelsStatus) {
                    eprintln!("\nErreur : {err:#}");
                }
                let _ = pause_enter("Entrée pour revenir au menu…");
            }
            Some(channels::ChannelMenuAction::Configure(id)) => {
                if let Err(err) = channels::run_channel_configure(workspace, id) {
                    eprintln!("\nErreur : {err:#}");
                }
                let _ = pause_enter("Entrée pour revenir au menu…");
            }
            None => {}
        }
    }
}

fn menu_short_title(id: SubMenuId) -> &'static str {
    match id {
        SubMenuId::SetupRoot => "setup",
        SubMenuId::Health => "santé",
        SubMenuId::Daemon => "daemon",
        SubMenuId::Gateway => "gateway",
        SubMenuId::GatewayChannels => "canaux",
        SubMenuId::Cortex => "cortex",
        SubMenuId::Drafts => "brouillons",
        SubMenuId::Skills => "skills",
        SubMenuId::SkillsHub => "hub",
        SubMenuId::SettingsRoot => "settings",
        SubMenuId::SettingsProviders => "providers",
        SubMenuId::Maintenance => "maintenance",
        SubMenuId::Watcher => "watcher",
    }
}