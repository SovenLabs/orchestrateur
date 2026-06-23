//! Stratos Quick Check — quatre blocs critiques pré-trade.

use crate::types::{ModuleId, QuickCheckBlock, QuickCheckItem, QuickCheckResult};

use super::context::ScoringContext;
use super::helpers::{
    block_passes, b12_validation, counter_trend, macro_climate, market_regime, session_active,
    structure_bias, tf_alignment_score,
};

/// Évalue le Quick Check Stratos.
#[must_use]
pub fn compute(ctx: &ScoringContext<'_>) -> QuickCheckResult {
    let macro_block = macro_block(ctx);
    let structure_block = structure_block(ctx);
    let liquidity_block = liquidity_of_block(ctx);
    let execution_block = execution_block(ctx);
    let blocks = vec![
        macro_block.clone(),
        structure_block.clone(),
        liquidity_block.clone(),
        execution_block.clone(),
    ];
    let passed = blocks.iter().all(|b| b.passed);
    let rationale = if passed {
        "Quick Check complet : macro, structure, liquidité/OF et exécution validés.".into()
    } else {
        let failed: Vec<&str> = blocks
            .iter()
            .filter(|b| !b.passed)
            .map(|b| b.name.as_str())
            .collect();
        format!(
            "Quick Check incomplet — blocs manquants : {}.",
            failed.join(", ")
        )
    };
    QuickCheckResult {
        passed,
        blocks,
        rationale,
    }
}

fn macro_block(ctx: &ScoringContext<'_>) -> QuickCheckBlock {
    let climate = macro_climate(ctx).as_str().to_string();
    let vol_ratio = ctx
        .module_payload(ModuleId::B1)
        .and_then(|p| p.get("volatility_ratio"))
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);
    let checks = vec![
        QuickCheckItem {
            label: "Climat macro non hostile".into(),
            passed: climate != "hostile",
        },
        QuickCheckItem {
            label: "Liquidité stable ou expansive".into(),
            passed: vol_ratio >= 0.65,
        },
        QuickCheckItem {
            label: "Sentiment risk-on ou neutre".into(),
            passed: climate == "favorable" || climate == "neutre",
        },
    ];
    QuickCheckBlock {
        name: "macro".into(),
        passed: block_passes(&checks),
        checks,
    }
}

fn structure_block(ctx: &ScoringContext<'_>) -> QuickCheckBlock {
    let htf_bias = ctx
        .module_payload(ModuleId::B2_5)
        .and_then(|p| p.get("htf_bias"))
        .and_then(|v| v.as_str())
        .unwrap_or("n/a");
    let bos = ctx
        .module_payload(ModuleId::B2)
        .and_then(|p| p.get("bos"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let bias = structure_bias(ctx).as_str().to_string();
    let regime = market_regime(ctx).as_str().to_string();
    let alignment = tf_alignment_score(ctx);
    let checks = vec![
        QuickCheckItem {
            label: "Trend HTF clair".into(),
            passed: htf_bias == "bull" || htf_bias == "bear",
        },
        QuickCheckItem {
            label: "BOS dans le sens du biais".into(),
            passed: bos
                && ((bias == "bull" && htf_bias != "bear")
                    || (bias == "bear" && htf_bias != "bull")),
        },
        QuickCheckItem {
            label: "Alignement HTF/LTF suffisant".into(),
            passed: alignment >= 55,
        },
        QuickCheckItem {
            label: "Pas de compression bloquante".into(),
            passed: regime != "compression",
        },
    ];
    QuickCheckBlock {
        name: "structure".into(),
        passed: block_passes(&checks),
        checks,
    }
}

fn liquidity_of_block(ctx: &ScoringContext<'_>) -> QuickCheckBlock {
    let pools = ctx
        .module_payload(ModuleId::B2)
        .and_then(|p| p.get("liquidity_pools"))
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let poc = ctx
        .module_payload(ModuleId::B12)
        .and_then(|p| p.get("poc"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let last_delta = ctx
        .module_payload(ModuleId::B12)
        .and_then(|p| p.get("last_delta"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let bias = structure_bias(ctx).as_str().to_string();
    let cvd_aligned = (bias == "bull" && last_delta >= 0.0)
        || (bias == "bear" && last_delta <= 0.0)
        || bias == "neutral";
    let absorption = ctx
        .module_payload(ModuleId::B12)
        .and_then(|p| p.get("absorption"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let validation = b12_validation(ctx).as_str().to_string();
    let checks = vec![
        QuickCheckItem {
            label: "Liquidité visible".into(),
            passed: pools > 0,
        },
        QuickCheckItem {
            label: "Sweep ou signal confirmé".into(),
            passed: ctx.any_signal_triggered(),
        },
        QuickCheckItem {
            label: "Zone VP pertinente".into(),
            passed: poc > 0.0,
        },
        QuickCheckItem {
            label: "CVD aligné ou absorption".into(),
            passed: cvd_aligned || absorption || validation == "absorption",
        },
    ];
    QuickCheckBlock {
        name: "liquidity_of".into(),
        passed: block_passes(&checks),
        checks,
    }
}

fn execution_block(ctx: &ScoringContext<'_>) -> QuickCheckBlock {
    let invalidation = ctx
        .module_payload(ModuleId::B2)
        .and_then(|p| p.get("invalidation"))
        .and_then(|v| v.as_f64())
        .is_some();
    let trade_exists = ctx
        .module_payload(ModuleId::B2)
        .and_then(|p| p.get("trade_exists"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let vol_ratio = ctx
        .module_payload(ModuleId::B1_5)
        .and_then(|p| p.get("volatility_ratio"))
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);
    let checks = vec![
        QuickCheckItem {
            label: "RR structurel ≥ 2 (proxy trade_exists)".into(),
            passed: trade_exists && !counter_trend(ctx),
        },
        QuickCheckItem {
            label: "Invalidation claire".into(),
            passed: invalidation,
        },
        QuickCheckItem {
            label: "Session active".into(),
            passed: session_active(ctx.session),
        },
        QuickCheckItem {
            label: "Volatilité suffisante".into(),
            passed: vol_ratio >= 0.5,
        },
    ];
    QuickCheckBlock {
        name: "execution".into(),
        passed: block_passes(&checks),
        checks,
    }
}