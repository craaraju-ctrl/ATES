use async_trait::async_trait;
use std::error::Error;
use ates_core::{Agent, AgentTier};

pub struct RiskCalculatorAgent;

#[async_trait]
impl Agent for RiskCalculatorAgent {
    fn name(&self) -> &str {
        "RiskCalculatorAgent"
    }

    fn tier(&self) -> AgentTier {
        AgentTier::Sub
    }

    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[{}] Calculating position size and checking risk limits...", self.name());
        Ok(())
    }
}