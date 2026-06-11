use async_trait::async_trait;
use std::error::Error;
use ates_core::{Agent, AgentTier, AgentInput, AgentOutput};

pub struct SessionTimerAgent;

#[async_trait]
impl Agent for SessionTimerAgent {
    fn name(&self) -> &str { "SessionTimerAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Sub }
    async fn run(&self, _input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        println!("[{}] Checking trading session...", self.name());
        Ok(AgentOutput::NoOutput)
    }
}