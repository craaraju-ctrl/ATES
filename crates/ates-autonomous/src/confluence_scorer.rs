use async_trait::async_trait;
use std::error::Error;
use ates_core::{Agent, AgentTier, AgentInput, AgentOutput, calculate_pivot_points, calculate_confluence_score};
use crate::state::SharedState;

pub struct ConfluenceScorerAgent {
    pub state: SharedState,
}

impl ConfluenceScorerAgent {
    pub fn new(state: SharedState) -> Self { Self { state }
    }
}

#[async_trait]
impl Agent for ConfluenceScorerAgent {
    fn name(&self) -> &str { "ConfluenceScorerAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Sub }

    async fn run(&self, input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        if let Some(AgentInput::ConfluenceRequest { context }) = input {
            let rules = self.state.rules.read().await;
            let pivots = calculate_pivot_points(context.high, context.low, context.previous_close, rules.pivot_method);
            let score = calculate_confluence_score(&context, &pivots);
            Ok(AgentOutput::ConfluenceResult(score))
        } else { Ok(AgentOutput::NoOutput) }
    }
}