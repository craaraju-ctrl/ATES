use async_trait::async_trait;
use std::error::Error;

use ates_core::{Agent, AgentTier};

pub struct OvertradingPreventerAgent;

#[async_trait]
impl Agent for OvertradingPreventerAgent {
    fn name(&self) -> &str {
        "OvertradingPreventerAgent"
    }

    fn tier(&self) -> AgentTier {
        AgentTier::Sub
    }

    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[{}] Checking consecutive losses and preventing overtrading...", self.name());
        // TODO: Use MemoryStore + max_consecutive_losses from DisciplineRules
        Ok(())
    }
}