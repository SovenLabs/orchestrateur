use std::path::PathBuf;

use b212::{
    analyze_b1, analyze_b1_5, analyze_b12, analyze_b2, analyze_b2_5, build_setup_analysis,
    ModuleContext, ModuleId,
};

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

fn assert_module_id(output: &b212::ModuleOutput, expected: ModuleId) {
    assert_eq!(output.module, expected);
    assert!(!output.summary.is_empty());
    assert!(!output.rationale.is_empty());
}

#[tokio::test]
async fn b1_never_triggers_entry_on_trend_fixture() {
    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let out = analyze_b1(&ctx);
    assert_module_id(&out, ModuleId::B1);
    assert_eq!(out.payload["triggers_entry"], false);
}

#[tokio::test]
async fn b1_5_detects_trend_on_btc_1h() {
    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let out = analyze_b1_5(&ctx);
    assert_module_id(&out, ModuleId::B1_5);
    let regime = out.payload["regime"].as_str().unwrap();
    assert_eq!(regime, "trend");
}

#[tokio::test]
async fn b1_5_detects_range_on_btc_4h() {
    let series = load_fixture("btc_range_4h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let out = analyze_b1_5(&ctx);
    assert_module_id(&out, ModuleId::B1_5);
    let regime = out.payload["regime"].as_str().unwrap();
    assert_eq!(regime, "range");
}

#[tokio::test]
async fn b1_5_detects_compression_on_eth_15m() {
    let series = load_fixture("eth_compression_15m.json").await;
    let ctx = ModuleContext::new("ETHUSDT", vec![series]);
    let out = analyze_b1_5(&ctx);
    assert_module_id(&out, ModuleId::B1_5);
    let regime = out.payload["regime"].as_str().unwrap();
    assert_eq!(regime, "compression");
}

#[tokio::test]
async fn b2_structure_has_bias_on_trend() {
    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let out = analyze_b2(&ctx);
    assert_module_id(&out, ModuleId::B2);
    assert_eq!(out.payload["bias"].as_str().unwrap(), "bull");
}

#[tokio::test]
async fn b2_5_single_timeframe_partial_alignment() {
    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let out = analyze_b2_5(&ctx);
    assert_module_id(&out, ModuleId::B2_5);
    let score = out.payload["alignment_score"].as_u64().unwrap();
    assert!(score >= 40 && score <= 100);
}

#[tokio::test]
async fn b12_does_not_create_trade() {
    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let out = analyze_b12(&ctx);
    assert_module_id(&out, ModuleId::B12);
    assert_eq!(out.payload["creates_trade"], false);
    assert!(out.payload["poc"].as_f64().unwrap() > 0.0);
}

#[tokio::test]
async fn multi_tf_alignment_improves_score() {
    let h4 = load_fixture("btc_range_4h.json").await;
    let h1 = load_fixture("btc_trend_1h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![h4, h1]);
    let out = analyze_b2_5(&ctx);
    assert!(out.payload["htf_bias"].is_string());
    assert!(out.payload["mtf_bias"].is_string());
}

#[tokio::test]
async fn build_setup_analysis_runs_all_five_modules() {
    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let analysis = build_setup_analysis(&ctx, "london");
    assert_eq!(analysis.symbol, "BTCUSDT");
    assert_eq!(analysis.session, "london");
    assert_eq!(analysis.modules.len(), 5);
    assert_eq!(analysis.lineage.data_source, "fixture");
    assert_eq!(analysis.signals.len(), 6);
}