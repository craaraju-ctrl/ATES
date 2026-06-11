use async_trait::async_trait;
use std::error::Error;

use ates_core::{Agent, AgentTier};

pub struct OutcomeLoggerAgent;

#[async_trait]
impl Agent for OutcomeLoggerAgent {
    fn name(&self) -> &str {
        "OutcomeLoggerAgent"
    }

    fn tier(&self) -> AgentTier {
        AgentTier::Sub
    }

    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[{}] Logging trade outcomes and decisions into MemoryStore...", self.name());
        // TODO: Create DecisionRecord and store via MemoryStore
        Ok(())
    }
}