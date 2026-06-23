//! Score d'alignement desk /10 (Bible section D).

use crate::types::{AlignmentScore, ModuleId};

use super::context::ScoringContext;
use super::helpers::{
    b12_validation, counter_trend, macro_climate, session_active, structure_bias,
    tf_alignment_score,
};

fn alignment_grade(total: u8) -> &'static str {
    if total >= 9 {
        "A_plus"
    } else if total >= 7 {
        "good"
    } else if total >= 5 {
        "medium"
    } else {
        "avoid"
    }
}

/// Calcule le score d'alignement desk.
#[must_use]
pub fn compute(ctx: &ScoringContext<'_>) -> AlignmentScore {
    let macro_score = score_macro(ctx);
    let structure_score = score_structure(ctx);
    let liquidity_score = score_liquidity(ctx);
    let derivatives_of_score = score_derivatives_of(ctx);
    let execution_score = score_execution(ctx);
    let total = macro_score
        .saturating_add(structure_score)
        .saturating_add(liquidity_score)
        .saturating_add(derivatives_of_score)
        .saturating_add(execution_score)
        .min(10);
    let grade = alignment_grade(total).to_string();
    let rationale = format!(
        "Alignement {total}/10 ({grade}) : macro {macro_score}, structure {structure_score}, \
         liquidité {liquidity_score}, dérivés/OF {derivatives_of_score}, exécution {execution_score}."
    );
    AlignmentScore {
        total,
        macro_score,
        structure_score,
        liquidity_score,
        derivatives_of_score,
        execution_score,
        grade,
        rationale,
    }
}

fn score_macro(ctx: &ScoringContext<'_>) -> u8 {
    match macro_climate(ctx).as_str() {
        "favorable" => 2,
        "neutre" => 1,
        _ => 0,
    }
}

fn score_structure(ctx: &ScoringContext<'_>) -> u8 {
    let bias = structure_bias(ctx).as_str().to_string();
    let trade_exists = ctx
        .module_payload(ModuleId::B2)
        .and_then(|p| p.get("trade_exists"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let bos = ctx
        .module_payload(ModuleId::B2)
        .and_then(|p| p.get("bos"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if trade_exists && (bias == "bull" || bias == "bear") {
        2
    } else if bos || bias != "neutral" {
        1
    } else {
        0
    }
}

fn score_liquidity(ctx: &ScoringContext<'_>) -> u8 {
    let pools = ctx
        .module_payload(ModuleId::B2)
        .and_then(|p| p.get("liquidity_pools"))
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    if pools > 0 && ctx.any_signal_triggered() {
        2
    } else if pools > 0 || ctx.any_signal_triggered() {
        1
    } else {
        0
    }
}

fn score_derivatives_of(ctx: &ScoringContext<'_>) -> u8 {
    match b12_validation(ctx).as_str() {
        "absorption" | "acceptance_above_value" | "rejection_below_value" => 2,
        "rotation_at_poc" => 1,
        _ => {
            let cvd = ctx
                .module_payload(ModuleId::B12)
                .and_then(|p| p.get("cvd"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            if cvd.abs() > 0.0 {
                1
            } else {
                0
            }
        }
    }
}

fn score_execution(ctx: &ScoringContext<'_>) -> u8 {
    let invalidation = ctx
        .module_payload(ModuleId::B2)
        .and_then(|p| p.get("invalidation"))
        .and_then(|v| v.as_f64())
        .is_some();
    let alignment = tf_alignment_score(ctx);
    let mut pts = 0u8;
    if !counter_trend(ctx) {
        pts = pts.saturating_add(1);
    }
    if invalidation {
        pts = pts.saturating_add(1);
    }
    if session_active(ctx.session) && alignment >= 55 {
        pts = pts.saturating_add(1);
    }
    pts.min(2)
}