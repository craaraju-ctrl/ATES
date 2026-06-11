use async_trait::async_trait;
use std::error::Error;
use ates_core::{Agent, AgentTier, AgentInput, AgentOutput};

pub struct OvertradingPreventerAgent;

#[async_trait]
impl Agent for OvertradingPreventerAgent {
    fn name(&self) -> &str { "OvertradingPreventerAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Sub }
    async fn run(&self, _input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        println!("[{}] Preventing overtrading...", self.name());
        Ok(AgentOutput::NoOutput)
    }
}