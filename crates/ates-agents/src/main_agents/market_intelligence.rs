use async_trait::async_trait;
use std::error::Error;
use ates_core::{Agent, AgentTier};

pub struct MarketIntelligenceAgent;

#[async_trait]
impl Agent for MarketIntelligenceAgent {
    fn name(&self) -> &str {
        "MarketIntelligenceAgent"
    }

    fn tier(&self) -> AgentTier {
        AgentTier::Main
    }

    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[{}] Gathering market data and calculating confluence...", self.name());
        Ok(())
    }
}