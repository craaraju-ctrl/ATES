use async_trait::async_trait;
use std::error::Error;
use ates_core::{Agent, AgentTier, AgentInput, AgentOutput};
use ates_autonomous::SharedState;

/// Delegates to `ates_autonomous::pivot_calculator::PivotCalculatorAgent`.
pub struct PivotCalculatorAgent {
    inner: ates_autonomous::pivot_calculator::PivotCalculatorAgent,
}

impl PivotCalculatorAgent {
    pub fn new(state: SharedState) -> Self {
        Self {
            inner: ates_autonomous::pivot_calculator::PivotCalculatorAgent::new(state),
        }
    }
}

#[async_trait]
impl Agent for PivotCalculatorAgent {
    fn name(&self) -> &str { self.inner.name() }
    fn tier(&self) -> AgentTier { self.inner.tier() }

    async fn run(&self, input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        self.inner.run(input).await
    }
}
