//! Vue graphe de connaissances.

use egui::Ui;
use orchestrator::HubSummary;

/// Affiche le résumé du graphe (nœuds, arêtes, hubs).
pub fn show_graph_view(
    ui: &mut Ui,
    node_count: usize,
    edge_count: usize,
    hubs: &[HubSummary],
    on_select: &mut dyn FnMut(&str),
) {
    ui.heading("Graphe de connaissances");
    ui.label(format!("Nœuds : {node_count}"));
    ui.label(format!("Arêtes : {edge_count}"));
    ui.separator();
    ui.heading("Hubs (backlinks entrants)");
    if hubs.is_empty() {
        ui.label("Aucun hub détecté.");
        return;
    }
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for hub in hubs {
                let label = format!(
                    "{} — {} lien(s) entrant(s)",
                    hub.title, hub.inbound_links
                );
                if ui.selectable_label(false, label).clicked() {
                    on_select(&hub.memory_id.to_string());
                }
            }
        });
}