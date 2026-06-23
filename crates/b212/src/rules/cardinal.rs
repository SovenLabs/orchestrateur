//! Règles cardinales Bible B212 (gouvernance pré-trade).

use crate::types::{
    B212SetupAnalysis, CardinalRuleId, CardinalRulesResult, CardinalViolation, ModuleId,
    SignalKind,
};

/// Évalue les règles cardinales sur une analyse complète.
#[must_use]
pub fn evaluate_cardinal_rules(analysis: &B212SetupAnalysis) -> CardinalRulesResult {
    let mut violations = Vec::new();

    check_context_never_triggers(analysis, &mut violations);
    check_execution_never_saves(analysis, &mut violations);
    check_flow_cannot_rescue_structure(analysis, &mut violations);
    check_structure_decides_trade(analysis, &mut violations);
    check_compression_not_accumulation(analysis, &mut violations);
    check_quick_check_complete(analysis, &mut violations);
    check_trade_location_minimum(analysis, &mut violations);
    check_narrative_auditable(analysis, &mut violations);

    let passed = violations.is_empty();
    let rationale = if passed {
        "Règles cardinales respectées — proposition autorisée si scores suffisants.".into()
    } else {
        format!(
            "{} violation(s) cardinale(s) — no trade.",
            violations.len()
        )
    };

    CardinalRulesResult {
        passed,
        violations,
        rationale,
    }
}

fn module_payload<'a>(
    analysis: &'a B212SetupAnalysis,
    id: ModuleId,
) -> Option<&'a serde_json::Value> {
    analysis
        .modules
        .iter()
        .find(|m| m.module == id)
        .map(|m| &m.payload)
}

fn trade_requested(analysis: &B212SetupAnalysis) -> bool {
    analysis
        .scores
        .as_ref()
        .is_some_and(|s| s.recommended_sizing != "none")
}

fn check_context_never_triggers(analysis: &B212SetupAnalysis, violations: &mut Vec<CardinalViolation>) {
    let triggers = module_payload(analysis, ModuleId::B1)
        .and_then(|p| p.get("triggers_entry"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if triggers {
        violations.push(CardinalViolation {
            rule: CardinalRuleId::ContextNeverTriggers,
            message: "B1 ne doit jamais déclencher d'entrée (triggers_entry=true).".into(),
        });
    }
}

fn check_structure_decides_trade(
    analysis: &B212SetupAnalysis,
    violations: &mut Vec<CardinalViolation>,
) {
    if !trade_requested(analysis) {
        return;
    }
    let trade_exists = module_payload(analysis, ModuleId::B2)
        .and_then(|p| p.get("trade_exists"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !trade_exists {
        violations.push(CardinalViolation {
            rule: CardinalRuleId::StructureDecidesTrade,
            message: "Taille demandée mais B2 confirme qu'aucun trade structurel n'existe.".into(),
        });
    }
}

fn check_execution_never_saves(analysis: &B212SetupAnalysis, violations: &mut Vec<CardinalViolation>) {
    let creates = module_payload(analysis, ModuleId::B12)
        .and_then(|p| p.get("creates_trade"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if creates {
        violations.push(CardinalViolation {
            rule: CardinalRuleId::ExecutionNeverSaves,
            message: "B12 ne crée jamais de trade (creates_trade=true interdit).".into(),
        });
    }
}

fn check_flow_cannot_rescue_structure(
    analysis: &B212SetupAnalysis,
    violations: &mut Vec<CardinalViolation>,
) {
    let trade_exists = module_payload(analysis, ModuleId::B2)
        .and_then(|p| p.get("trade_exists"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if trade_exists {
        return;
    }
    let validation = module_payload(analysis, ModuleId::B12)
        .and_then(|p| p.get("validation"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let strong_flow = matches!(
        validation,
        "absorption" | "acceptance_above_value" | "rejection_below_value"
    );
    if strong_flow {
        violations.push(CardinalViolation {
            rule: CardinalRuleId::FlowCannotRescueStructure,
            message: format!(
                "Flow fort ({validation}) sans structure valide — le flow ne sauve pas B2."
            ),
        });
    }
}

fn check_compression_not_accumulation(
    analysis: &B212SetupAnalysis,
    violations: &mut Vec<CardinalViolation>,
) {
    let regime = module_payload(analysis, ModuleId::B1_5)
        .and_then(|p| p.get("regime"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if regime != "compression" {
        return;
    }
    let bias = module_payload(analysis, ModuleId::B2)
        .and_then(|p| p.get("bias"))
        .and_then(|v| v.as_str())
        .unwrap_or("neutral");
    if bias != "bull" || !trade_requested(analysis) {
        return;
    }
    let acceptance = analysis
        .signals
        .iter()
        .find(|s| s.kind == SignalKind::AcceptanceExpansion)
        .is_some_and(|s| s.triggered);
    if !acceptance {
        violations.push(CardinalViolation {
            rule: CardinalRuleId::CompressionNotAccumulation,
            message: "Compression + biais bull : pas d'Acceptance Expansion — accumulation non garantie."
                .into(),
        });
    }
}

fn check_quick_check_complete(
    analysis: &B212SetupAnalysis,
    violations: &mut Vec<CardinalViolation>,
) {
    if !trade_requested(analysis) {
        return;
    }
    let passed = analysis
        .scores
        .as_ref()
        .is_some_and(|s| s.quick_check.passed);
    if !passed {
        violations.push(CardinalViolation {
            rule: CardinalRuleId::QuickCheckComplete,
            message: "Quick Check incomplet — no trade ou observation seule.".into(),
        });
    }
}

fn check_trade_location_minimum(
    analysis: &B212SetupAnalysis,
    violations: &mut Vec<CardinalViolation>,
) {
    if !trade_requested(analysis) {
        return;
    }
    let tls = analysis
        .scores
        .as_ref()
        .map(|s| s.trade_location.total)
        .unwrap_or(0);
    if tls < 6 {
        violations.push(CardinalViolation {
            rule: CardinalRuleId::TradeLocationMinimum,
            message: format!("TLS {tls}/10 < 6 — emplacement insuffisant."),
        });
    }
}

fn check_narrative_auditable(analysis: &B212SetupAnalysis, violations: &mut Vec<CardinalViolation>) {
    let expected = [
        ModuleId::B1,
        ModuleId::B1_5,
        ModuleId::B2,
        ModuleId::B2_5,
        ModuleId::B12,
    ];
    for id in expected {
        let Some(module) = analysis.modules.iter().find(|m| m.module == id) else {
            violations.push(CardinalViolation {
                rule: CardinalRuleId::NarrativeAuditable,
                message: format!("Module {id:?} absent — narratif non auditable."),
            });
            continue;
        };
        if module.summary.is_empty() || module.rationale.is_empty() {
            violations.push(CardinalViolation {
                rule: CardinalRuleId::NarrativeAuditable,
                message: format!("Module {id:?} incomplet — 6 phrases Bible requises."),
            });
        }
    }
}