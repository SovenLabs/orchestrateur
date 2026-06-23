//! Menus interactifs harness (`setup`, `settings`, `onboard`).

mod actions;
mod menus;
mod onboard;
mod theme;

use std::path::Path;

use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Select};

use menus::{MenuAction, SubMenuId};
use theme::{format_menu_line, pause_enter, print_banner, print_breadcrumb, print_footer};

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
        let title = menus::title_for(current);
        print_banner(title, workspace);

        let crumbs: Vec<&str> = stack.iter().map(|id| menu_short_title(*id)).collect();
        print_breadcrumb(&crumbs);

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