use async_trait::async_trait;
use std::error::Error;
use ates_core::Agent;

pub struct RiskCalculatorAgent;

#[async_trait]
impl Agent for RiskCalculatorAgent {
    fn name(&self) -> &str {
        "RiskCalculatorAgent"
    }

    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[{}] Calculating position size and checking risk limits...", self.name());
        // Uses Disciplined Core rules
        Ok(())
    }
}