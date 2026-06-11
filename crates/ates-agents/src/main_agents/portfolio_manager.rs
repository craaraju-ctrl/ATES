use async_trait::async_trait;
use std::error::Error;
use ates_core::{Agent, AgentTier, AgentInput, AgentOutput};

pub struct PortfolioManagerAgent;

#[async_trait]
impl Agent for PortfolioManagerAgent {
    fn name(&self) -> &str { "PortfolioManagerAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Main }
    async fn run(&self, _input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        println!("[{}] Managing portfolio risk...", self.name());
        Ok(AgentOutput::NoOutput)
    }
}