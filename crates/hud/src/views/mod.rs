//! Vues HUD v0.4 — bannière dégradée, graphe, audit.

mod audit_view;
mod graph_view;
mod health_banner;

pub use audit_view::show_audit_view;
pub use graph_view::show_graph_view;
pub use health_banner::show_degraded_banner;