use async_trait::async_trait;
use std::error::Error;
use ates_core::{Agent, AgentTier, AgentInput, AgentOutput};

pub struct RedFolderCheckerAgent;

#[async_trait]
impl Agent for RedFolderCheckerAgent {
    fn name(&self) -> &str { "RedFolderCheckerAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Sub }
    async fn run(&self, _input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        println!("[{}] Checking red folder...", self.name());
        Ok(AgentOutput::NoOutput)
    }
}