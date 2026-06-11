use async_trait::async_trait;
use std::error::Error;
use ates_core::{Agent, AgentTier, AgentInput, AgentOutput};

pub struct ReflectorAgent;

#[async_trait]
impl Agent for ReflectorAgent {
    fn name(&self) -> &str { "ReflectorAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Main }
    async fn run(&self, _input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        println!("[{}] Reflecting on past decisions...", self.name());
        Ok(AgentOutput::NoOutput)
    }
}