use async_trait::async_trait;
use std::error::Error;

use ates_core::{Agent, AgentTier};

pub struct ExecutionCoordinatorAgent;

#[async_trait]
impl Agent for ExecutionCoordinatorAgent {
    fn name(&self) -> &str {
        "ExecutionCoordinatorAgent"
    }

    fn tier(&self) -> AgentTier {
        AgentTier::Main
    }

    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[{}] Coordinating order execution through BrokerAdapter and ExecutionEngine...", self.name());
        Ok(())
    }
}