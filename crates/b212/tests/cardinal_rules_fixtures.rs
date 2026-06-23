use std::path::PathBuf;

use b212::{build_setup_analysis, evaluate_cardinal_rules, CardinalRuleId, ModuleContext};

async fn load_fixture(name: &str) -> b212::OhlcvSeries {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("workspace")
        .join("b212")
        .join("fixtures")
        .join(name);
    let raw = tokio::fs::read_to_string(&path).await.unwrap();
    serde_json::from_str(&raw).unwrap()
}

#[tokio::test]
async fn build_setup_analysis_includes_cardinal_rules() {
    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let analysis = build_setup_analysis(&ctx, "london");
    let cardinal = analysis.cardinal.expect("cardinal rules populated");
    assert!(!cardinal.rationale.is_empty());
}

#[tokio::test]
async fn b1_never_triggers_entry_rule_passes() {
    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let analysis = build_setup_analysis(&ctx, "london");
    let cardinal = evaluate_cardinal_rules(&analysis);
    assert!(
        !cardinal
            .violations
            .iter()
            .any(|v| v.rule == CardinalRuleId::ContextNeverTriggers)
    );
}

#[tokio::test]
async fn b12_never_creates_trade_rule_passes() {
    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let analysis = build_setup_analysis(&ctx, "london");
    let cardinal = evaluate_cardinal_rules(&analysis);
    assert!(
        !cardinal
            .violations
            .iter()
            .any(|v| v.rule == CardinalRuleId::ExecutionNeverSaves)
    );
}

#[tokio::test]
async fn narrative_auditable_rule_passes_on_complete_pipeline() {
    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let analysis = build_setup_analysis(&ctx, "london");
    let cardinal = evaluate_cardinal_rules(&analysis);
    assert!(
        !cardinal
            .violations
            .iter()
            .any(|v| v.rule == CardinalRuleId::NarrativeAuditable)
    );
}

#[tokio::test]
async fn compression_fixture_may_violate_quick_check_when_sizing_requested() {
    let series = load_fixture("eth_compression_15m.json").await;
    let ctx = ModuleContext::new("ETHUSDT", vec![series]);
    let analysis = build_setup_analysis(&ctx, "asia");
    let cardinal = analysis.cardinal.unwrap();
    if analysis.scores.as_ref().is_some_and(|s| s.recommended_sizing != "none") {
        let has_qc_or_tls = cardinal.violations.iter().any(|v| {
            matches!(
                v.rule,
                CardinalRuleId::QuickCheckComplete | CardinalRuleId::TradeLocationMinimum
            )
        });
        assert!(
            has_qc_or_tls || !cardinal.passed,
            "compression with sizing should fail QC or TLS: {:?}",
            cardinal.violations
        );
    }
}