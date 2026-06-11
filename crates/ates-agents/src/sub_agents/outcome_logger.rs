use async_trait::async_trait;
use std::error::Error;

use ates_core::{Agent, AgentTier, AgentInput, AgentOutput, MemoryStore};

pub struct OutcomeLoggerAgent;

#[async_trait]
impl Agent for OutcomeLoggerAgent {
    fn name(&self) -> &str { "OutcomeLoggerAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Sub }

    async fn run(&self, input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        match input {
            Some(AgentInput::LogOutcome { key, value }) => {
                if let Ok(memory) = MemoryStore::new("ates_memory.redb") {
                    let _ = memory.store_decision(&key, &value);
                }
                Ok(AgentOutput::Done)
            }
            _ => Ok(AgentOutput::NoOutput),
        }
    }
}