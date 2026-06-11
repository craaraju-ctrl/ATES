use async_trait::async_trait;
use std::error::Error;
use chrono::Utc;
use ates_core::{Agent, AgentTier, AgentInput, AgentOutput};
use crate::state::SharedState;

pub struct ExecutionCoordinatorAgent {
    pub state: SharedState,
}

impl ExecutionCoordinatorAgent {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }

    async fn execute_paper_trade(&self, signal: &crate::types::TradeSignal) -> Result<String, Box<dyn Error + Send + Sync>> {
        println!(
            "[ExecutionCoordinator] Executing paper trade: {} {} @ {:.2} | Qty: {:.0}",
            signal.symbol,
            if signal.direction == ates_core::TradeDirection::Long { "BUY" } else { "SELL" },
            signal.entry_price, signal.position_size
        );

        println!("[ExecutionCoordinator] Paper order filled (simulated)");
        Ok(format!("Paper trade executed: {}", signal.symbol))
    }
}

#[async_trait]
impl Agent for ExecutionCoordinatorAgent {
    fn name(&self) -> &str { "ExecutionCoordinatorAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Main }

    async fn run(&self, input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        println!("[ExecutionCoordinatorAgent] Managing order lifecycle...");
        Ok(AgentOutput::Done)
    }
}