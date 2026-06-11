use async_trait::async_trait;
use std::error::Error;

use ates_core::{Agent, AgentTier, AgentInput, AgentOutput, PivotMethod, calculate_pivot_points};

pub struct PivotCalculatorAgent;

#[async_trait]
impl Agent for PivotCalculatorAgent {
    fn name(&self) -> &str { "PivotCalculatorAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Sub }

    async fn run(&self, input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        match input {
            Some(AgentInput::PivotRequest { high, low, close }) => {
                let pivots = calculate_pivot_points(high, low, close, PivotMethod::Classic);
                Ok(AgentOutput::PivotResult(pivots))
            }
            _ => Ok(AgentOutput::NoOutput),
        }
    }
}