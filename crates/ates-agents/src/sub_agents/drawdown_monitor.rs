use async_trait::async_trait;
use std::error::Error;

use ates_core::{Agent, AgentTier};

pub struct DrawdownMonitorAgent;

#[async_trait]
impl Agent for DrawdownMonitorAgent {
    fn name(&self) -> &str {
        "DrawdownMonitorAgent"
    }

    fn tier(&self) -> AgentTier {
        AgentTier::Sub
    }

    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[{}] Monitoring daily drawdown and enforcing max drawdown limits...", self.name());
        // TODO: Integrate with MemoryStore and check_risk_limits
        Ok(())
    }
}