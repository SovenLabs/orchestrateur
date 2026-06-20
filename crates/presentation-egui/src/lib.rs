//! # Presentation egui — La Peau (optionnelle)
//!
//! Phase 6 : HUD minimal (liste mémoires, recherche, détail).
//! Build sélectif : `cargo build -p presentation-egui`

#![allow(dead_code)]

/// Version du crate alignée sur le workspace.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Fenêtre HUD — squelette Phase 0.
pub struct HudShell;