//! Signaux avancés B212 (Bible section VII).

mod acceptance_expansion;
mod cascade_trigger;
mod context;
mod false_migration;
mod helpers;
mod impulse_trigger;
mod leverage_trap;
mod value_migration;

pub use acceptance_expansion::evaluate as evaluate_acceptance_expansion;
pub use cascade_trigger::evaluate as evaluate_cascade_trigger;
pub use context::SignalContext;
pub use false_migration::evaluate as evaluate_false_migration;
pub use impulse_trigger::evaluate as evaluate_impulse_trigger;
pub use leverage_trap::evaluate as evaluate_leverage_trap;
pub use value_migration::evaluate as evaluate_value_migration;

use crate::types::SignalOutput;

/// Évalue les 6 signaux avancés dans l'ordre canonique.
#[must_use]
pub fn run_all_signals(sig_ctx: &SignalContext<'_>) -> Vec<SignalOutput> {
    vec![
        evaluate_value_migration(sig_ctx),
        evaluate_acceptance_expansion(sig_ctx),
        evaluate_false_migration(sig_ctx),
        evaluate_impulse_trigger(sig_ctx),
        evaluate_cascade_trigger(sig_ctx),
        evaluate_leverage_trap(sig_ctx),
    ]
}