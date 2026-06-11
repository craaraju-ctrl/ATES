use async_trait::async_trait;
use std::error::Error;
use ates_core::{Agent, AgentTier, AgentInput, AgentOutput};

pub struct DrawdownMonitorAgent;

#[async_trait]
impl Agent for DrawdownMonitorAgent {
    fn name(&self) -> &str { "DrawdownMonitorAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Sub }
    async fn run(&self, _input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        println!("[{}] Monitoring drawdown...", self.name());
        Ok(AgentOutput::NoOutput)
    }
}