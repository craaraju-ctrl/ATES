use async_trait::async_trait;
use std::error::Error;
use ates_core::{Agent, AgentTier, AgentInput, AgentOutput};

pub struct RiskCalculatorAgent;

#[async_trait]
impl Agent for RiskCalculatorAgent {
    fn name(&self) -> &str { "RiskCalculatorAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Sub }
    async fn run(&self, _input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        println!("[{}] Calculating risk...", self.name());
        Ok(AgentOutput::NoOutput)
    }
}