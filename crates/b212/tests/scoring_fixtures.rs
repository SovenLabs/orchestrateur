use std::path::PathBuf;

use b212::{
    build_score_bundle, build_setup_analysis, run_all, run_all_signals, ModuleContext, ScoringContext,
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

fn scoring_ctx<'a>(
    ctx: &'a ModuleContext,
    modules: &'a [b212::ModuleOutput],
    signals: &'a [b212::SignalOutput],
    session: &'a str,
) -> ScoringContext<'a> {
    ScoringContext {
        ctx,
        modules,
        signals,
        session,
    }
}

#[tokio::test]
async fn build_setup_analysis_includes_score_bundle() {
    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let analysis = build_setup_analysis(&ctx, "london");
    let scores = analysis.scores.expect("scores should be populated");
    assert!(scores.trade_location.total <= 10);
    assert!(scores.alignment.total <= 10);
    assert_eq!(scores.quick_check.blocks.len(), 4);
    assert!(!scores.recommended_sizing.is_empty());
}

#[tokio::test]
async fn trend_fixture_trade_location_score_in_range() {
    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let modules = run_all(&ctx);
    let signals = run_all_signals(&b212::SignalContext {
        ctx: &ctx,
        modules: &modules,
    });
    let score_ctx = scoring_ctx(&ctx, &modules, &signals, "london");
    let bundle = build_score_bundle(&score_ctx);
    assert!(
        bundle.trade_location.total >= 3,
        "trend fixture TLS too low: {:?}",
        bundle.trade_location
    );
    assert!(
        bundle.alignment.total >= 4,
        "trend fixture alignment too low: {:?}",
        bundle.alignment
    );
}

#[tokio::test]
async fn compression_fixture_quick_check_may_fail_structure() {
    let series = load_fixture("eth_compression_15m.json").await;
    let ctx = ModuleContext::new("ETHUSDT", vec![series]);
    let modules = run_all(&ctx);
    let signals = run_all_signals(&b212::SignalContext {
        ctx: &ctx,
        modules: &modules,
    });
    let score_ctx = scoring_ctx(&ctx, &modules, &signals, "asia");
    let bundle = build_score_bundle(&score_ctx);
    let structure = bundle
        .quick_check
        .blocks
        .iter()
        .find(|b| b.name == "structure")
        .expect("structure block");
    assert!(
        !structure.passed,
        "compression should fail structure quick check: {:?}",
        structure.checks
    );
}

#[tokio::test]
async fn tls_sizing_follows_bible_thresholds() {
    let series = load_fixture("btc_trend_1h.json").await;
    let ctx = ModuleContext::new("BTCUSDT", vec![series]);
    let analysis = build_setup_analysis(&ctx, "london");
    let tls = &analysis.scores.unwrap().trade_location;
    let expected = if tls.total >= 8 {
        "normal"
    } else if tls.total >= 6 {
        "reduced"
    } else {
        "none"
    };
    assert_eq!(tls.sizing, expected);
}

#[test]
fn score_bundle_roundtrip_json() {
    let bundle = b212::ScoreBundle {
        trade_location: b212::TradeLocationScore {
            total: 7,
            liquidity_proximity: 2,
            htf_ltf_confluence: 2,
            extreme_or_retest: 1,
            b12_validation: 2,
            sizing: "reduced".into(),
            rationale: "test".into(),
        },
        quick_check: b212::QuickCheckResult {
            passed: true,
            blocks: vec![b212::QuickCheckBlock {
                name: "macro".into(),
                passed: true,
                checks: vec![b212::QuickCheckItem {
                    label: "ok".into(),
                    passed: true,
                }],
            }],
            rationale: "ok".into(),
        },
        alignment: b212::AlignmentScore {
            total: 8,
            macro_score: 2,
            structure_score: 2,
            liquidity_score: 1,
            derivatives_of_score: 2,
            execution_score: 1,
            grade: "good".into(),
            rationale: "test".into(),
        },
        recommended_sizing: "reduced".into(),
    };
    let json = serde_json::to_string(&bundle).unwrap();
    let back: b212::ScoreBundle = serde_json::from_str(&json).unwrap();
    assert_eq!(back, bundle);
}