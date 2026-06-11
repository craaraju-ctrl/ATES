use async_trait::async_trait;
use std::error::Error;
use ates_core::{Agent, AgentTier, AgentInput, AgentOutput};

pub struct PatternRetrieverAgent;

#[async_trait]
impl Agent for PatternRetrieverAgent {
    fn name(&self) -> &str { "PatternRetrieverAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Sub }
    async fn run(&self, _input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        println!("[{}] Retrieving patterns...", self.name());
        Ok(AgentOutput::NoOutput)
    }
}