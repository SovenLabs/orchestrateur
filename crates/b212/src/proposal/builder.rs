//! Construction de propositions trade depuis une analyse.

use chrono::Utc;
use uuid::Uuid;

use crate::error::B212Error;
use crate::rules::evaluate_cardinal_rules;
use crate::types::{B212SetupAnalysis, ModuleId, ProposalStatus, TradeProposal};

/// Construit le narratif 6 phrases Bible.
#[must_use]
pub fn build_narrative(analysis: &B212SetupAnalysis) -> String {
    let find = |id: ModuleId| {
        analysis
            .modules
            .iter()
            .find(|m| m.module == id)
            .map(|m| m.summary.as_str())
            .unwrap_or("n/a")
    };
    let (sizing, tls, alignment) = analysis
        .scores
        .as_ref()
        .map(|s| {
            (
                s.recommended_sizing.as_str(),
                s.trade_location.total,
                s.alignment.total,
            )
        })
        .unwrap_or(("none", 0, 0));
    format!(
        "1. Macro : {} 2. Régime : {} 3. Structure : {} 4. Alignement : {} \
         5. Validation B12 : {} 6. Plan : taille {sizing}, TLS {tls}/10, alignement {alignment}/10.",
        find(ModuleId::B1),
        find(ModuleId::B1_5),
        find(ModuleId::B2),
        find(ModuleId::B2_5),
        find(ModuleId::B12),
    )
}

/// Détermine le sens (`long`, `short`, `observe`).
#[must_use]
pub fn determine_side(analysis: &B212SetupAnalysis) -> String {
    let sizing = analysis
        .scores
        .as_ref()
        .map(|s| s.recommended_sizing.as_str())
        .unwrap_or("none");
    if sizing == "none" {
        return "observe".into();
    }
    let bias = analysis
        .modules
        .iter()
        .find(|m| m.module == ModuleId::B2)
        .and_then(|m| m.payload.get("bias"))
        .and_then(|v| v.as_str())
        .unwrap_or("neutral");
    match bias {
        "bull" => "long".into(),
        "bear" => "short".into(),
        _ => "observe".into(),
    }
}

/// Crée une proposition HITL si les règles cardinales et scores le permettent.
///
/// # Errors
///
/// Retourne [`B212Error::CardinalRules`] si une règle cardinale est violée.
pub fn build_trade_proposal(analysis: &B212SetupAnalysis) -> Result<TradeProposal, B212Error> {
    let cardinal = analysis
        .cardinal
        .as_ref()
        .cloned()
        .unwrap_or_else(|| evaluate_cardinal_rules(analysis));
    if !cardinal.passed {
        let summary: Vec<String> = cardinal
            .violations
            .iter()
            .map(|v| format!("{:?}: {}", v.rule, v.message))
            .collect();
        return Err(B212Error::CardinalRules(summary.join("; ")));
    }

    let scores = analysis.scores.as_ref().ok_or_else(|| {
        B212Error::ProposalBlocked("scores manquants — pipeline incomplet".into())
    })?;

    let side = determine_side(analysis);
    if side == "observe" {
        return Err(B212Error::ProposalBlocked(
            "observation seule — sizing none ou biais neutre".into(),
        ));
    }

    let id = format!("b212-{}", Uuid::now_v7());
    Ok(TradeProposal {
        id,
        symbol: analysis.symbol.clone(),
        session: analysis.session.clone(),
        side,
        status: ProposalStatus::PendingHuman,
        trade_location_score: scores.trade_location.total,
        quick_check_passed: scores.quick_check.passed,
        alignment_score: scores.alignment.total,
        sizing: scores.recommended_sizing.clone(),
        narrative: build_narrative(analysis),
        lineage: analysis.lineage.clone(),
        created_at: Utc::now().to_rfc3339(),
        reject_reason: None,
    })
}