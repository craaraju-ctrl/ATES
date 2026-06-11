use async_trait::async_trait;
use std::error::Error;
use ates_core::{Agent, AgentTier, AgentInput, AgentOutput};

pub struct StrategyDecisionAgent;

#[async_trait]
impl Agent for StrategyDecisionAgent {
    fn name(&self) -> &str { "StrategyDecisionAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Main }
    async fn run(&self, _input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        println!("[{}] Making final decisions...", self.name());
        Ok(AgentOutput::NoOutput)
    }
}