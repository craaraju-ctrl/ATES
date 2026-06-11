use async_trait::async_trait;
use std::error::Error;

use ates_core::{Agent, AgentTier};

pub struct ConfluenceScorerAgent;

#[async_trait]
impl Agent for ConfluenceScorerAgent {
    fn name(&self) -> &str {
        "ConfluenceScorerAgent"
    }

    fn tier(&self) -> AgentTier {
        AgentTier::Sub
    }

    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[{}] Calculating confluence score using pivots, trend, and volume...", self.name());
        // TODO: Use calculate_confluence_score from disciplined_core
        Ok(())
    }
}