//! Liste virtualisée manuelle — rendu O(visible) pour 5 000+ mémoires.

use egui::{Pos2, Rect, Sense, Ui, Vec2};
use orchestrator::MemorySummary;

/// Hauteur fixe estimée par ligne (titre + tags sur une ligne).
pub const ROW_HEIGHT: f32 = 32.0;

/// Calcule l'intervalle de lignes visibles pour la virtualisation.
#[must_use]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn visible_row_range(
    total_rows: usize,
    clip_min_y: f32,
    clip_max_y: f32,
    content_min_y: f32,
    row_height: f32,
) -> (usize, usize) {
    if total_rows == 0 {
        return (0, 0);
    }
    let start_row = ((clip_min_y - content_min_y) / row_height)
        .floor()
        .max(0.0) as usize;
    let end_row = (((clip_max_y - content_min_y) / row_height).ceil() as usize).min(total_rows);
    (start_row, end_row)
}

/// Affiche une liste virtualisée de [`MemorySummary`].
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn show_virtual_memory_list(
    ui: &mut Ui,
    memories: &[MemorySummary],
    selected_id: Option<&str>,
    on_select: &mut impl FnMut(&MemorySummary),
) {
    if memories.is_empty() {
        ui.label("Aucune mémoire — assimilez du contenu via la barre de recherche.");
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            let total_rows = memories.len();
            let total_height = total_rows as f32 * ROW_HEIGHT;
            let (content_rect, _) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), total_height), Sense::hover());

            let clip = ui.clip_rect();
            let (start_row, end_row) = visible_row_range(
                total_rows,
                clip.min.y,
                clip.max.y,
                content_rect.min.y,
                ROW_HEIGHT,
            );

            for (row, memory) in memories
                .iter()
                .enumerate()
                .skip(start_row)
                .take(end_row.saturating_sub(start_row))
            {
                let y = content_rect.min.y + row as f32 * ROW_HEIGHT;
                let row_rect = Rect::from_min_size(
                    Pos2::new(content_rect.min.x, y),
                    Vec2::new(content_rect.width(), ROW_HEIGHT),
                );
                let id = memory.id.to_string();
                let selected = selected_id == Some(id.as_str());

                #[allow(deprecated)]
                ui.allocate_ui_at_rect(row_rect, |ui| {
                    ui.horizontal(|ui| {
                        let label = if memory.tags.is_empty() {
                            memory.title.clone()
                        } else {
                            format!("{}  [{}]", memory.title, memory.tags.join(", "))
                        };
                        let response = ui.selectable_label(selected, label);
                        if response.clicked() {
                            on_select(memory);
                        }
                    });
                });
            }
        });
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    #[test]
    fn visible_range_covers_viewport() {
        let (start, end) = visible_row_range(5000, 100.0, 400.0, 0.0, ROW_HEIGHT);
        assert!(end > start);
        assert!(end - start <= 20);
    }

    #[test]
    fn visible_range_5k_rows_computes_under_budget() {
        let start = Instant::now();
        for _ in 0..10_000 {
            let _ = visible_row_range(5000, 120.0, 520.0, 0.0, ROW_HEIGHT);
        }
        assert!(
            start.elapsed() < Duration::from_millis(50),
            "calcul range trop lent: {:?}",
            start.elapsed()
        );
    }

    #[test]
    fn visible_range_empty_for_zero_rows() {
        assert_eq!(visible_row_range(0, 0.0, 100.0, 0.0, ROW_HEIGHT), (0, 0));
    }
}