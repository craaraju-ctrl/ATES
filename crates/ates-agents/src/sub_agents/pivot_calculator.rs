use async_trait::async_trait;
use std::error::Error;

use ates_core::{Agent, AgentTier};

pub struct PivotCalculatorAgent;

#[async_trait]
impl Agent for PivotCalculatorAgent {
    fn name(&self) -> &str {
        "PivotCalculatorAgent"
    }

    fn tier(&self) -> AgentTier {
        AgentTier::Sub
    }

    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[{}] Calculating pivot points using Disciplined Core...", self.name());

        // Example calculation (in real flow this would come from market data)
        let example_pivots = ates_core::calculate_pivot_points(24500.0, 24200.0, 24350.0, ates_core::PivotMethod::Classic);
        println!(
            "   Pivot: {:.2} | R1: {:.2} | S1: {:.2}",
            example_pivots.pivot, example_pivots.r1, example_pivots.s1
        );

        Ok(())
    }
}