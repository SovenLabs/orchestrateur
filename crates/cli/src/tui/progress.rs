//! Barre de progression terminal.

use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use console::style;

/// Session de progression avec libellé et pourcentage.
pub struct ProgressSession {
    title: String,
    status: String,
    percent: u8,
}

impl ProgressSession {
    /// Démarre une session (« Starting Orchestrateur… »).
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            status: String::new(),
            percent: 0,
        }
    }

    /// Met à jour le statut et le pourcentage (0–100).
    pub fn set(&mut self, status: impl Into<String>, percent: u8) {
        self.status = status.into();
        self.percent = percent.min(100);
        self.render();
    }

    /// Incrémente avec animation courte.
    pub fn tick(&mut self, status: impl Into<String>, percent: u8) {
        self.set(status, percent);
        thread::sleep(Duration::from_millis(120));
    }

    fn render(&self) {
        let width = 32usize;
        let filled = (usize::from(self.percent) * width) / 100;
        let bar: String = (0..width)
            .map(|i| if i < filled { '█' } else { '░' })
            .collect();
        let line = format!(
            "\r{} {} [{}] {}%",
            style(&self.title).bold(),
            style(&bar).cyan(),
            style(self.percent).dim(),
            self.percent
        );
        print!("{line}");
        let _ = io::stdout().flush();
    }

    /// Termine avec message de succès ou avertissement.
    pub fn finish(mut self, message: impl Into<String>, ok: bool) -> Result<()> {
        self.set(message, 100);
        println!();
        if ok {
            println!("{}", style("✓ Harness prêt").green());
        } else {
            println!("{}", style("⚠ Harness partiellement prêt — voir ci-dessous").yellow());
        }
        Ok(())
    }
}

/// Affiche une carte d'intro (texte centré visuellement).
pub fn print_setup_card(headline: &str, subtitle: &str) {
    let line = "─".repeat(54);
    println!();
    println!("{}", style(&line).dim());
    println!("{}", style(headline).bold());
    println!("{}", style(subtitle).dim());
    println!("{}", style(&line).dim());
    println!();
}