use async_trait::async_trait;
use std::error::Error;

use ates_core::{Agent, AgentTier};

pub struct PortfolioManagerAgent;

#[async_trait]
impl Agent for PortfolioManagerAgent {
    fn name(&self) -> &str {
        "PortfolioManagerAgent"
    }

    fn tier(&self) -> AgentTier {
        AgentTier::Main
    }

    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[{}] Managing overall portfolio risk, position sizing across symbols...", self.name());
        Ok(())
    }
}