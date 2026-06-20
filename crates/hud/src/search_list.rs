//! Liste virtualisée des résultats de recherche.

use egui::{Pos2, Rect, Sense, Ui, Vec2};

use crate::list::visible_row_range;
use crate::state::SearchHitView;

/// Hauteur par hit (score + extrait optionnel).
pub const SEARCH_ROW_HEIGHT: f32 = 44.0;

/// Affiche les hits de recherche de façon virtualisée.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn show_virtual_search_list(
    ui: &mut Ui,
    hits: &[SearchHitView],
    selected_id: Option<&str>,
    on_select: &mut impl FnMut(&SearchHitView),
) {
    if hits.is_empty() {
        ui.label("Aucun résultat — élargissez la requête.");
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            let total_rows = hits.len();
            let total_height = total_rows as f32 * SEARCH_ROW_HEIGHT;
            let (content_rect, _) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), total_height), Sense::hover());

            let clip = ui.clip_rect();
            let (start_row, end_row) = visible_row_range(
                total_rows,
                clip.min.y,
                clip.max.y,
                content_rect.min.y,
                SEARCH_ROW_HEIGHT,
            );

            for (row, hit) in hits
                .iter()
                .enumerate()
                .skip(start_row)
                .take(end_row.saturating_sub(start_row))
            {
                let y = content_rect.min.y + row as f32 * SEARCH_ROW_HEIGHT;
                let row_rect = Rect::from_min_size(
                    Pos2::new(content_rect.min.x, y),
                    Vec2::new(content_rect.width(), SEARCH_ROW_HEIGHT),
                );
                let selected = selected_id == Some(hit.id.as_str());

                #[allow(deprecated)]
                ui.allocate_ui_at_rect(row_rect, |ui| {
                    ui.vertical(|ui| {
                        let score_pct = (hit.score.clamp(0.0, 1.0) * 100.0) as u32;
                        let title = hit
                            .snippet
                            .as_deref()
                            .unwrap_or(&hit.id);
                        let label = format!("{score_pct}% — {title}");
                        let response = ui.selectable_label(selected, label);
                        if response.clicked() {
                            on_select(hit);
                        }
                    });
                });
            }
        });
}