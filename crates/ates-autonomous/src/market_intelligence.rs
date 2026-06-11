use async_trait::async_trait;
use std::error::Error;
use chrono::Utc;
use ates_core::{
    Agent, AgentTier, AgentInput, AgentOutput,
    MarketContext, calculate_pivot_points, calculate_confluence_score,
    is_in_trading_session,
};
use crate::state::SharedState;

pub struct MarketIntelligenceAgent {
    pub state: SharedState,
}

impl MarketIntelligenceAgent {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }

    async fn analyze_market(&self, symbol: &str, price: f64) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        let rules = self.state.rules.read().await;

        let high = price * 1.015;
        let low = price * 0.985;
        let prev_close = price * 0.998;

        let context = MarketContext {
            symbol: symbol.to_string(),
            current_price: price,
            high,
            low,
            previous_close: prev_close,
            timestamp: Utc::now(),
            daily_pnl: 0.0,
            consecutive_losses: 0,
            is_red_folder_day: false,
            trend_direction: None,
        };

        let pivots = calculate_pivot_points(high, low, prev_close, rules.pivot_method);
        let confluence = calculate_confluence_score(&context, &pivots);
        let session_valid = is_in_trading_session(Utc::now(), &rules);

        println!(
            "[MarketIntelligence] {} @ {:.2} | Pivot: {:.2} | R1: {:.2} | S1: {:.2} | Confluence: {:.2}% | Session: {}",
            symbol, price, pivots.pivot, pivots.r1, pivots.s1, confluence * 100.0,
            if session_valid { "VALID" } else { "INVALID" }
        );

        let _ = self.state.memory.store_decision(
            &format!("market/{}/{}", symbol, Utc::now().timestamp()),
            &format!("price={:.2},confluence={:.3}", price, confluence),
        );

        {
            let mut regime = self.state.market_regime.write().await;
            *regime = Some(if confluence > 0.7 {
                crate::types::MarketRegime::TrendingBull
            } else if confluence < 0.4 {
                crate::types::MarketRegime::TrendingBear
            } else {
                crate::types::MarketRegime::Ranging
            });
        }

        Ok(AgentOutput::ConfluenceResult(confluence))
    }
}

#[async_trait]
impl Agent for MarketIntelligenceAgent {
    fn name(&self) -> &str { "MarketIntelligenceAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Main }

    async fn run(&self, input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        match input {
            Some(AgentInput::ConfluenceRequest { context }) => {
                self.analyze_market(&context.symbol, context.current_price).await
            }
            Some(AgentInput::PivotRequest { high, low, close }) => {
                let rules = self.state.rules.read().await;
                let pivots = calculate_pivot_points(high, low, close, rules.pivot_method);
                println!("[MarketIntelligence] Pivot levels calculated on request");
                Ok(AgentOutput::PivotResult(pivots))
            }
            _ => self.analyze_market("NIFTY", 24500.0).await,
        }
    }
}