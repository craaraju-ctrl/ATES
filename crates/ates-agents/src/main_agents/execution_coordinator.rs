use async_trait::async_trait;
use std::error::Error;
use ates_core::{Agent, AgentTier, AgentInput, AgentOutput};

pub struct ExecutionCoordinatorAgent;

impl ExecutionCoordinatorAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Agent for ExecutionCoordinatorAgent {
    fn name(&self) -> &str { "ExecutionCoordinatorAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Main }

    async fn run(&self, _input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        Ok(AgentOutput::NoOutput)
    }
}
