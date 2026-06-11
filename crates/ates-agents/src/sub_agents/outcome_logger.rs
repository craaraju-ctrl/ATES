use async_trait::async_trait;
use std::error::Error;

use ates_core::{Agent, AgentTier};

pub struct OutcomeLoggerAgent;

#[async_trait]
impl Agent for OutcomeLoggerAgent {
    fn name(&self) -> &str {
        "OutcomeLoggerAgent"
    }

    fn tier(&self) -> AgentTier {
        AgentTier::Sub
    }

    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[{}] Logging trade outcomes and decisions into MemoryStore...", self.name());

        // Demo: Create a memory store and log a sample decision
        if let Ok(memory) = ates_core::MemoryStore::new("ates_memory.redb") {
            let _ = memory.store_decision(
                "demo-decision-001",
                r#"{"symbol":"NIFTY","direction":"Long","pnl":1250.5,"confluence":0.87}"#,
            );
            println!("   Decision logged to MemoryStore.");
        }

        Ok(())
    }
}