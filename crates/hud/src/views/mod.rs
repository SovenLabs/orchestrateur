//! Vues HUD — bannière dégradée, graphe, audit, chat.

mod audit_view;
mod chat_view;
mod graph_view;
mod health_banner;

pub use audit_view::show_audit_view;
pub use chat_view::show_chat_view;
pub use graph_view::show_graph_view;
pub use health_banner::show_degraded_banner;