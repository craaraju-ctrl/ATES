use async_trait::async_trait;
use std::error::Error;

use ates_core::{Agent, AgentTier, AgentInput, AgentOutput, calculate_pivot_points, calculate_confluence_score};

pub struct ConfluenceScorerAgent;

#[async_trait]
impl Agent for ConfluenceScorerAgent {
    fn name(&self) -> &str { "ConfluenceScorerAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Sub }

    async fn run(&self, input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        match input {
            Some(AgentInput::ConfluenceRequest { context }) => {
                let pivots = calculate_pivot_points(context.high, context.low, context.previous_close, ates_core::PivotMethod::Classic);
                let score = calculate_confluence_score(&context, &pivots);
                Ok(AgentOutput::ConfluenceResult(score))
            }
            _ => Ok(AgentOutput::NoOutput),
        }
    }
}