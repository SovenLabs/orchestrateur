use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use b212::{B212Error, Bar, MarketDataProvider, MarketScenario, OhlcvSeries, Timeframe};
use serde::Deserialize;

/// Charge des séries OHLCV depuis des fichiers JSON (`workspace/b212/fixtures/`).
pub struct FixtureMarketDataProvider {
    fixtures_dir: PathBuf,
    index: HashMap<(String, Timeframe), PathBuf>,
}

#[derive(Debug, Deserialize)]
struct FixtureFile {
    symbol: String,
    timeframe: TimeframeWire,
    scenario: MarketScenario,
    bars: Vec<Bar>,
}

#[derive(Debug, Deserialize)]
enum TimeframeWire {
    #[serde(alias = "m15", alias = "15m")]
    M15,
    #[serde(alias = "h1", alias = "1h")]
    H1,
    #[serde(alias = "h4", alias = "4h")]
    H4,
    #[serde(alias = "d1", alias = "1d")]
    D1,
}

impl From<TimeframeWire> for Timeframe {
    fn from(value: TimeframeWire) -> Self {
        match value {
            TimeframeWire::M15 => Self::M15,
            TimeframeWire::H1 => Self::H1,
            TimeframeWire::H4 => Self::H4,
            TimeframeWire::D1 => Self::D1,
        }
    }
}

impl FixtureMarketDataProvider {
    /// Crée le provider et indexe les fixtures connues.
    #[must_use]
    pub fn new(fixtures_dir: impl Into<PathBuf>) -> Self {
        let fixtures_dir = fixtures_dir.into();
        let mut index = HashMap::new();
        index.insert(
            ("BTCUSDT".into(), Timeframe::H1),
            fixtures_dir.join("btc_trend_1h.json"),
        );
        index.insert(
            ("BTCUSDT".into(), Timeframe::H4),
            fixtures_dir.join("btc_range_4h.json"),
        );
        index.insert(
            ("ETHUSDT".into(), Timeframe::M15),
            fixtures_dir.join("eth_compression_15m.json"),
        );
        Self {
            fixtures_dir,
            index,
        }
    }

    /// Répertoire des fixtures.
    #[must_use]
    pub fn fixtures_dir(&self) -> &Path {
        &self.fixtures_dir
    }

    fn resolve_path(&self, symbol: &str, timeframe: Timeframe) -> Result<&PathBuf, B212Error> {
        let key = (symbol.to_uppercase(), timeframe);
        self.index.get(&key).ok_or_else(|| {
            B212Error::FixtureNotFound {
                path: format!(
                    "{} (symbol={symbol}, tf={})",
                    self.fixtures_dir.display(),
                    timeframe.label()
                ),
            }
        })
    }

    async fn load_file(path: &Path) -> Result<OhlcvSeries, B212Error> {
        let raw = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| B212Error::FixtureNotFound {
                path: format!("{}: {e}", path.display()),
            })?;
        let parsed: FixtureFile = serde_json::from_str(&raw)
            .map_err(|e| B212Error::Parse(format!("{}: {e}", path.display())))?;
        Ok(OhlcvSeries {
            symbol: parsed.symbol,
            timeframe: parsed.timeframe.into(),
            scenario: parsed.scenario,
            bars: parsed.bars,
        })
    }
}

#[async_trait]
impl MarketDataProvider for FixtureMarketDataProvider {
    async fn get_ohlcv(
        &self,
        symbol: &str,
        timeframe: Timeframe,
        lookback: usize,
    ) -> Result<OhlcvSeries, B212Error> {
        let path = self.resolve_path(symbol, timeframe)?;
        let series = Self::load_file(path).await?;
        if lookback == 0 {
            return Ok(series);
        }
        if series.bars.len() < lookback {
            return Err(B212Error::InsufficientBars {
                need: lookback,
                got: series.bars.len(),
            });
        }
        Ok(series.tail(lookback))
    }
}