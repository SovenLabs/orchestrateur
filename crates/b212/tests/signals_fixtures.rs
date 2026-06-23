use std::path::PathBuf;

use b212::{
    build_setup_analysis, run_all, run_all_signals, ModuleContext, SignalContext, SignalKind,
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

fn signal_kinds(outputs: &[b212::SignalOutput]) -> Vec<SignalKind> {
    outputs.iter().map(|s| s.kind).collect()
}

#[tokio::test]
async fn run_all_signals_returns_six_outputs() {
    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let modules = run_all(&ctx);
    let sig_ctx = SignalContext {
        ctx: &ctx,
        modules: &modules,
    };
    let signals = run_all_signals(&sig_ctx);
    assert_eq!(signals.len(), 6);
    assert_eq!(
        signal_kinds(&signals),
        vec![
            SignalKind::ValueMigration,
            SignalKind::AcceptanceExpansion,
            SignalKind::FalseMigrationTrap,
            SignalKind::ImpulseTrigger,
            SignalKind::CascadeTrigger,
            SignalKind::LeverageTrap,
        ]
    );
    for sig in &signals {
        assert!(!sig.rationale.is_empty());
        assert_eq!(sig.lineage.data_source, "fixture");
    }
}

#[tokio::test]
async fn trend_fixture_triggers_value_migration_or_cascade() {
    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let analysis = build_setup_analysis(&ctx, "london");
    assert_eq!(analysis.signals.len(), 6);
    let any_triggered = analysis.signals.iter().any(|s| s.triggered);
    assert!(
        any_triggered,
        "trend fixture should trigger at least one signal: {:?}",
        analysis
            .signals
            .iter()
            .map(|s| (s.kind, s.score, s.triggered))
            .collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn compression_fixture_scores_impulse_trigger() {
    let series = load_fixture("eth_compression_15m.json").await;
    let ctx = ModuleContext::new("ETHUSDT", vec![series]);
    let modules = run_all(&ctx);
    let sig_ctx = SignalContext {
        ctx: &ctx,
        modules: &modules,
    };
    let signals = run_all_signals(&sig_ctx);
    let impulse = signals
        .iter()
        .find(|s| s.kind == SignalKind::ImpulseTrigger)
        .unwrap();
    assert!(
        impulse.score >= 25,
        "compression should satisfy at least 1 impulse condition: {}",
        impulse.rationale
    );
}

#[tokio::test]
async fn range_fixture_may_trigger_false_migration() {
    let series = load_fixture("btc_range_4h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let analysis = build_setup_analysis(&ctx, "ny");
    let false_mig = analysis
        .signals
        .iter()
        .find(|s| s.kind == SignalKind::FalseMigrationTrap)
        .unwrap();
    assert!(!false_mig.rationale.is_empty());
}

#[tokio::test]
async fn build_setup_analysis_includes_modules_and_signals() {
    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let analysis = build_setup_analysis(&ctx, "london");
    assert_eq!(analysis.modules.len(), 5);
    assert_eq!(analysis.signals.len(), 6);
    assert_eq!(analysis.lineage.loader, "b212_pipeline");
}

#[test]
fn signal_output_roundtrip_json() {
    let sig = b212::SignalOutput {
        kind: SignalKind::ValueMigration,
        score: 72,
        triggered: true,
        rationale: "test".into(),
        lineage: b212::B212Lineage::fixture("test"),
    };
    let json = serde_json::to_string(&sig).unwrap();
    let back: b212::SignalOutput = serde_json::from_str(&json).unwrap();
    assert_eq!(back, sig);
}