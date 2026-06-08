use tokio::task::JoinSet;
use std::error::Error;

use ates_core::{DisciplineRules, get_discipline_summary};

struct MarketAgent;
struct RiskAgent;

#[async_trait::async_trait]
impl ates_core::Agent for MarketAgent {
    fn name(&self) -> &str { "MarketAgent" }
    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[{}] Running market analysis...", self.name());
        Ok(())
    }
}

#[async_trait::async_trait]
impl ates_core::Agent for RiskAgent {
    fn name(&self) -> &str { "RiskAgent" }
    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[{}] Checking risk rules...", self.name());
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    println!("=== ATES Orchestrator ===");
    println!("{}", get_discipline_summary());

    let _rules = DisciplineRules::default();

    let mut set = JoinSet::new();
    set.spawn(async move { MarketAgent.run().await });
    set.spawn(async move { RiskAgent.run().await });

    while let Some(_) = set.join_next().await {}

    println!("Cycle completed.");
}