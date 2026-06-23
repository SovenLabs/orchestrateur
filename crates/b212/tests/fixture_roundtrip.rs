use b212::{MarketScenario, Timeframe};

#[test]
fn timeframe_labels_are_stable() {
    assert_eq!(Timeframe::H1.label(), "1h");
    assert_eq!(Timeframe::H4.label(), "4h");
    assert_eq!(Timeframe::M15.label(), "15m");
}

#[test]
fn market_scenarios_serialize() {
    let json = serde_json::to_string(&MarketScenario::Trend).unwrap();
    assert_eq!(json, "\"trend\"");
}