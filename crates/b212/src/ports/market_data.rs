use async_trait::async_trait;

use crate::error::B212Error;
use crate::types::{OhlcvSeries, Timeframe};

/// Port de données marché — fixtures Phase 3, adapters live ultérieurs.
#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    /// Charge une série OHLCV pour un symbole et une unité de temps.
    async fn get_ohlcv(
        &self,
        symbol: &str,
        timeframe: Timeframe,
        lookback: usize,
    ) -> Result<OhlcvSeries, B212Error>;
}