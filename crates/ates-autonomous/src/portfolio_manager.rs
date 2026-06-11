use async_trait::async_trait;
use std::error::Error;
use chrono::Utc;
use ates_core::{Agent, AgentTier, AgentInput, AgentOutput};
use crate::state::SharedState;

pub struct PortfolioManagerAgent {
    pub state: SharedState,
}

impl PortfolioManagerAgent {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }

    async fn assess_portfolio(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let portfolio = self.state.portfolio.read().await;

        let total_exposure: f64 = portfolio.open_positions.iter()
            .map(|p| p.quantity * p.current_price)
            .sum();

        let exposure_pct = if portfolio.total_equity > 0.0 {
            total_exposure / portfolio.total_equity * 100.0
        } else { 0.0 };

        let assessment = format!(
            "Portfolio: Equity ₹{:.2} | Cash ₹{:.2} | Exposure {:.1}% | Positions {} | Today P&L: ₹{:.2}",
            portfolio.total_equity, portfolio.cash_balance, exposure_pct,
            portfolio.open_positions.len(), portfolio.daily_pnl
        );

        println!("[PortfolioManager] {}", assessment);
        Ok(assessment)
    }
}

#[async_trait]
impl Agent for PortfolioManagerAgent {
    fn name(&self) -> &str { "PortfolioManagerAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Main }

    async fn run(&self, _input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        let _ = self.assess_portfolio().await;
        Ok(AgentOutput::Done)
    }
}