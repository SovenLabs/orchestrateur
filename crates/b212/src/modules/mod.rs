//! Modules Bible B212 (B1, B1.5, B2, B2.5, B12).

mod b1_5_regime;
mod b1_macro;
mod b12_orderflow;
mod b2_5_timeframes;
mod b2_structure;
pub(crate) mod common;
mod context;

pub use b1_5_regime::{analyze as analyze_b1_5, MarketRegime};
pub use b1_macro::{analyze as analyze_b1, MacroClimate};
pub use b12_orderflow::analyze as analyze_b12;
pub use b2_5_timeframes::analyze as analyze_b2_5;
pub use b2_structure::{analyze as analyze_b2, StructureBias};
pub use context::ModuleContext;

use crate::signals::{run_all_signals, SignalContext};
use crate::types::{B212Lineage, B212SetupAnalysis, ModuleOutput};

/// Exécute tous les modules B212 dans l'ordre canonique.
#[must_use]
pub fn run_all(ctx: &ModuleContext) -> Vec<ModuleOutput> {
    vec![
        analyze_b1(ctx),
        analyze_b1_5(ctx),
        analyze_b2(ctx),
        analyze_b2_5(ctx),
        analyze_b12(ctx),
    ]
}

/// Agrège les sorties modules en analyse setup complète (PR-4 enrichira scores).
#[must_use]
pub fn build_setup_analysis(ctx: &ModuleContext, session: &str) -> B212SetupAnalysis {
    let modules = run_all(ctx);
    let sig_ctx = SignalContext {
        ctx,
        modules: &modules,
    };
    let signals = run_all_signals(&sig_ctx);
    B212SetupAnalysis {
        symbol: ctx.symbol.clone(),
        session: session.to_string(),
        modules,
        signals,
        scores: None,
        lineage: B212Lineage::fixture("b212_pipeline"),
    }
}