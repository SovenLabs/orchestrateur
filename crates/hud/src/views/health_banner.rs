//! Bannière permanente en mode dégradé (embeddings indisponibles).

use egui::{Color32, Context, RichText, TopBottomPanel};

/// Affiche une bannière persistante lorsque la recherche sémantique est indisponible.
pub fn show_degraded_banner(ctx: &Context, embedding_available: bool, llm_available: bool) {
    if embedding_available && llm_available {
        return;
    }

    TopBottomPanel::top("degraded_banner").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let mut parts = Vec::new();
            if !embedding_available {
                parts.push("recherche sémantique indisponible");
            }
            if !llm_available {
                parts.push("assimilation LLM indisponible");
            }
            let message = format!(
                "Mode dégradé — {} — liste, détail, graphe et audit restent disponibles",
                parts.join(" · ")
            );
            ui.colored_label(
                Color32::from_rgb(220, 140, 60),
                RichText::new(message).strong(),
            );
        });
    });
}