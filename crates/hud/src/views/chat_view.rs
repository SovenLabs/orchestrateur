//! Vue chat libre avec le provider LLM.

use egui::Ui;

/// Affiche la saisie chat et la dernière réponse.
pub fn show_chat_view(
    ui: &mut Ui,
    input: &mut String,
    reply: Option<&str>,
    llm_available: bool,
    on_send: &mut dyn FnMut(),
) {
    ui.heading("Chat LLM");
    if !llm_available {
        ui.colored_label(
            egui::Color32::from_rgb(220, 80, 80),
            "Chat indisponible — provider LLM hors ligne",
        );
        return;
    }
    ui.label("Message :");
    ui.text_edit_multiline(input);
    if ui.button("Envoyer").clicked() && !input.trim().is_empty() {
        on_send();
    }
    ui.separator();
    ui.heading("Réponse");
    if let Some(text) = reply {
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.label(text);
            });
    } else {
        ui.label("Aucune réponse pour l'instant.");
    }
}