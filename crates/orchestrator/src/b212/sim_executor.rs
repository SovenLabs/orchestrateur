//! SimExecutor paper — fill fixture + persistance + transition HITL.

use std::sync::Arc;

use b212::{
    execute_paper_fill, B212Error, MarketDataProvider, SimFill, SimTradeRepository, Timeframe,
    TradeProposal,
};

use super::governance::B212GovernanceService;

/// Service d'exécution paper B212.
pub struct B212SimExecutorService {
    governance: B212GovernanceService,
    market_data: Arc<dyn MarketDataProvider>,
    sim_trades: Arc<dyn SimTradeRepository>,
    base_notional_usd: f64,
}

impl B212SimExecutorService {
    /// Construit le service avec gouvernance, marché et dépôt fills.
    pub fn new(
        governance: B212GovernanceService,
        market_data: Arc<dyn MarketDataProvider>,
        sim_trades: Arc<dyn SimTradeRepository>,
        base_notional_usd: f64,
    ) -> Self {
        Self {
            governance,
            market_data,
            sim_trades,
            base_notional_usd,
        }
    }

    /// Exécute une proposition approuvée en paper : fill + persistance + journal.
    pub async fn execute(&self, id: &str) -> Result<(TradeProposal, SimFill), B212Error> {
        let proposal = self.governance.get_proposal(id).await?;
        let entry_price = self.resolve_entry_price(&proposal).await?;
        let fill = execute_paper_fill(&proposal, entry_price, self.base_notional_usd)?;
        self.sim_trades.save(&fill).await?;
        let executed = self.governance.sim_execute_with_fill(id, Some(&fill)).await?;
        Ok((executed, fill))
    }

    async fn resolve_entry_price(&self, proposal: &TradeProposal) -> Result<f64, B212Error> {
        let symbol = proposal.symbol.to_uppercase();
        for tf in [Timeframe::H1, Timeframe::H4, Timeframe::M15] {
            if let Ok(series) = self.market_data.get_ohlcv(&symbol, tf, 0).await {
                if let Some(price) = series.last_close() {
                    return Ok(price);
                }
            }
        }
        Err(B212Error::MarketData(format!(
            "aucun prix fixture pour {}",
            proposal.symbol
        )))
    }
}