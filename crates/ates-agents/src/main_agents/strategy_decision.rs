use async_trait::async_trait;
use std::error::Error;

use ates_core::{Agent, AgentTier};

pub struct StrategyDecisionAgent;

#[async_trait]
impl Agent for StrategyDecisionAgent {
    fn name(&self) -> &str {
        "StrategyDecisionAgent"
    }

    fn tier(&self) -> AgentTier {
        AgentTier::Main
    }

    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[{}] Making final trade direction decisions based on all agent inputs + Disciplined Core...", self.name());
        // TODO: Coordinate outputs from MarketIntelligence + RiskPsychology + Sub-Agents
        Ok(())
    }
}