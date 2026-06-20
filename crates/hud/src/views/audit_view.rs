//! Vue journal d'audit.

use egui::{Color32, Ui};
use orchestrator::AuditEvent;

/// Affiche les entrées d'audit et le statut de la chaîne BLAKE3.
pub fn show_audit_view(ui: &mut Ui, entries: &[AuditEvent], chain_intact: bool) {
    ui.heading("Journal d'audit");
    if chain_intact {
        ui.colored_label(Color32::from_rgb(80, 180, 100), "Chaîne BLAKE3 : intacte");
    } else {
        ui.colored_label(Color32::from_rgb(220, 80, 80), "Chaîne BLAKE3 : ROMPUE");
    }
    ui.separator();
    if entries.is_empty() {
        ui.label("Aucune entrée d'audit.");
        return;
    }
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for entry in entries {
                ui.group(|ui| {
                    ui.label(format!("{} — {}", entry.timestamp, entry.event_type));
                    ui.label(&entry.details);
                    ui.monospace(format!("hash: {}", entry.hash));
                });
            }
        });
}