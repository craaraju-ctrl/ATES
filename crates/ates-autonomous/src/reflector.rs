use async_trait::async_trait;
use std::error::Error;
use chrono::Utc;
use ates_core::{Agent, AgentTier, AgentInput, AgentOutput};
use crate::state::SharedState;

pub struct ReflectorAgent {
    pub state: SharedState,
}

impl ReflectorAgent {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }

    async fn reflect(&self, symbol: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        println!("[Reflector] Reflecting on past decisions for {}...", symbol);

        let today_key = format!("decisions/{}/{}", symbol, Utc::now().format("%Y%m%d"));
        let recent = self.state.memory.get_decision(&today_key).ok().flatten();

        let pattern_key = format!("patterns/{}", symbol);
        let patterns = self.state.memory.get_decision(&pattern_key).ok().flatten();

        let portfolio = self.state.portfolio.read().await;

        let reflection = format!(
            "Reflection for {}: Daily P&L: ₹{:.2} | Trades: {} | Wins: {} | Losses: {} | Consecutive Losses: {}",
            symbol, portfolio.daily_pnl, portfolio.total_trades_today,
            portfolio.winning_trades_today, portfolio.losing_trades_today,
            portfolio.consecutive_losses
        );

        println!("[Reflector] {}", reflection);

        let reflection_key = format!("reflections/{}/{}", symbol, Utc::now().timestamp());
        let _ = self.state.memory.store_decision(&reflection_key, &reflection);

        Ok(reflection)
    }
}

#[async_trait]
impl Agent for ReflectorAgent {
    fn name(&self) -> &str { "ReflectorAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Main }

    async fn run(&self, input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        let symbol = match &input {
            Some(AgentInput::ConfluenceRequest { context }) => context.symbol.clone(),
            _ => "NIFTY".to_string(),
        };

        let _ = self.reflect(&symbol).await;
        Ok(AgentOutput::Done)
    }
}