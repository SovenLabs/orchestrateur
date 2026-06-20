//! Thème egui dark / light minimal.

use egui::{Context, Visuals};

/// Applique le thème sombre ou clair sur le contexte egui.
pub fn apply_theme(ctx: &Context, dark_mode: bool) {
    if dark_mode {
        ctx.set_visuals(Visuals::dark());
    } else {
        ctx.set_visuals(Visuals::light());
    }
}