//! Icône application embarquée (32×32 RGBA).

use eframe::egui;

const SIZE: u32 = 32;

/// Génère l'icône « Orchestrateur » (cercle central sur fond sombre).
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn app_icon() -> egui::IconData {
    let mut rgba = vec![0_u8; (SIZE * SIZE * 4) as usize];
    let center = SIZE as f32 / 2.0 - 0.5;

    for y in 0..SIZE {
        for x in 0..SIZE {
            let idx = ((y * SIZE + x) * 4) as usize;
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let dist = (dx * dx + dy * dy).sqrt();
            let (r, g, b) = if dist < 9.0 {
                (74_u8, 158, 255)
            } else if dist < 11.0 {
                (40, 80, 140)
            } else {
                (18, 18, 32)
            };
            rgba[idx] = r;
            rgba[idx + 1] = g;
            rgba[idx + 2] = b;
            rgba[idx + 3] = 255;
        }
    }

    egui::IconData {
        width: SIZE,
        height: SIZE,
        rgba,
    }
}