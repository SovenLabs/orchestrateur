//! Bannière, formatage hints et pause post-action.

use std::io::{self, Write};
use std::path::Path;

use anyhow::Result;
use console::{style, Term};
use orchestrator::VERSION;

/// Affiche la bannière du menu harness.
pub fn print_banner(title: &str, workspace: &Path) {
    let line = "─".repeat(52);
    println!();
    println!("{}", style(&line).dim());
    println!(
        "{}  {}",
        style("Orchestrateur").bold().cyan(),
        style(format!("v{VERSION}")).dim()
    );
    println!("{}  {}", style(title).bold(), style(workspace.display()).dim());
    println!("{}", style(&line).dim());
    println!();
}

/// Fil d'Ariane au-dessus de la liste.
pub fn print_breadcrumb(crumbs: &[&str]) {
    if crumbs.is_empty() {
        return;
    }
    let trail: Vec<String> = crumbs.iter().map(|c| (*c).to_string()).collect();
    println!("{}", style(trail.join(" › ")).dim());
    println!();
}

/// Libellé affiché dans dialoguer : `Label (hint)`.
pub fn format_menu_line(label: &str, hint: &str, badge: Option<&str>) -> String {
    let mut line = format!("{} ({})", label, hint);
    if let Some(b) = badge {
        line.push(' ');
        line.push_str(b);
    }
    line
}

/// Badges d'état (`[actif]`, `[needs setup]`).
pub fn print_status_chips(entries: &[(&str, &str)]) {
    if entries.is_empty() {
        return;
    }
    let line: Vec<String> = entries
        .iter()
        .map(|(name, badge)| format!("{} {}", style(name).dim(), style(badge).yellow()))
        .collect();
    println!("{}", line.join("  ·  "));
    println!();
}

/// Pied de page raccourcis clavier.
pub fn print_footer() {
    println!();
    println!(
        "{}",
        style("↑↓ naviguer · Entrée valider · q quitter").dim()
    );
}

/// Pause avant retour au menu.
pub fn pause_enter(message: &str) -> Result<()> {
    let term = Term::stdout();
    write!(io::stdout(), "\n{message}")?;
    io::stdout().flush()?;
    term.read_line()?;
    Ok(())
}