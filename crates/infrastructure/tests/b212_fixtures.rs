use std::path::PathBuf;

use b212::{MarketDataProvider, MarketScenario, Timeframe};
use infrastructure::FixtureMarketDataProvider;

fn workspace_fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("workspace")
        .join("b212")
        .join("fixtures")
}

#[tokio::test]
async fn btc_trend_1h_fixture_loads() {
    let provider = FixtureMarketDataProvider::new(workspace_fixtures());
    let series = provider
        .get_ohlcv("BTCUSDT", Timeframe::H1, 10)
        .await
        .unwrap();
    assert_eq!(series.symbol, "BTCUSDT");
    assert_eq!(series.timeframe, Timeframe::H1);
    assert_eq!(series.scenario, MarketScenario::Trend);
    assert_eq!(series.bars.len(), 10);
    let first = series.bars.first().unwrap().close;
    let last = series.bars.last().unwrap().close;
    assert!(last > first, "trend fixture should rise");
}

#[tokio::test]
async fn btc_range_4h_fixture_loads() {
    let provider = FixtureMarketDataProvider::new(workspace_fixtures());
    let series = provider
        .get_ohlcv("BTCUSDT", Timeframe::H4, 12)
        .await
        .unwrap();
    assert_eq!(series.scenario, MarketScenario::Range);
    assert_eq!(series.bars.len(), 12);
}

#[tokio::test]
async fn eth_compression_15m_fixture_loads() {
    let provider = FixtureMarketDataProvider::new(workspace_fixtures());
    let series = provider
        .get_ohlcv("ETHUSDT", Timeframe::M15, 8)
        .await
        .unwrap();
    assert_eq!(series.scenario, MarketScenario::Compression);
    assert_eq!(series.bars.len(), 8);
    let spread = series.bars.last().unwrap().high - series.bars.last().unwrap().low;
    assert!(spread < 5.0, "compression: range should narrow");
}

#[tokio::test]
async fn unknown_symbol_returns_fixture_not_found() {
    let provider = FixtureMarketDataProvider::new(workspace_fixtures());
    let err = provider
        .get_ohlcv("SOLUSDT", Timeframe::H1, 5)
        .await
        .unwrap_err();
    assert!(matches!(err, b212::B212Error::FixtureNotFound { .. }));
}