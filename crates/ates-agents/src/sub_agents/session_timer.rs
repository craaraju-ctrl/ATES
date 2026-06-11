use async_trait::async_trait;
use std::error::Error;

use ates_core::{Agent, AgentTier};

pub struct SessionTimerAgent;

#[async_trait]
impl Agent for SessionTimerAgent {
    fn name(&self) -> &str {
        "SessionTimerAgent"
    }

    fn tier(&self) -> AgentTier {
        AgentTier::Sub
    }

    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[{}] Checking if current time is within London/NY trading sessions...", self.name());
        // TODO: Use is_in_trading_session from disciplined_core
        Ok(())
    }
}