use async_trait::async_trait;
use std::error::Error;
use ates_core::{Agent, AgentTier, AgentInput, AgentOutput};
use ates_autonomous::SharedState;

/// Delegates to `ates_autonomous::drawdown_monitor::DrawdownMonitorAgent`.
pub struct DrawdownMonitorAgent {
    inner: ates_autonomous::drawdown_monitor::DrawdownMonitorAgent,
}

impl DrawdownMonitorAgent {
    pub fn new(state: SharedState) -> Self {
        Self {
            inner: ates_autonomous::drawdown_monitor::DrawdownMonitorAgent::new(state),
        }
    }
}

#[async_trait]
impl Agent for DrawdownMonitorAgent {
    fn name(&self) -> &str { self.inner.name() }
    fn tier(&self) -> AgentTier { self.inner.tier() }

    async fn run(&self, input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        self.inner.run(input).await
    }
}
