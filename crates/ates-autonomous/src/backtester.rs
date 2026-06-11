// backtester.rs
// Implementation of AutonomousBacktester for simulated historical runs

use crate::orchestrator_struct::AutonomousOrchestrator;
use ates_core::{MarketContext, TradeDirection};
use std::error::Error;

pub struct AutonomousBacktester {
    pub orchestrator: AutonomousOrchestrator,
}

impl AutonomousBacktester {
    pub fn new(orchestrator: AutonomousOrchestrator) -> Self {
        Self { orchestrator }
    }

    pub async fn run_backtest(
        &self,
        symbol: &str,
        direction: TradeDirection,
        data: Vec<MarketContext>,
    ) -> Result<AutonomousBacktestResult, Box<dyn Error + Send + Sync>> {
        let mut summaries = Vec::new();
        for ctx in data {
            let summary = self.orchestrator.run_full_pipeline(
                symbol,
                direction,
                ctx.current_price,
                ctx.current_price * 0.99,
                ctx.current_price * 1.02,
            ).await?;
            summaries.push(summary);
        }

        let total_runs = summaries.len();
        let executed_count = summaries.iter().filter(|s| s.executed).count();

        Ok(AutonomousBacktestResult {
            total_runs,
            executed_count,
            message: format!("Autonomous backtest finished. Runs: {}, Executed: {}", total_runs, executed_count),
        })
    }
}

#[derive(Debug, Clone)]
pub struct AutonomousBacktestResult {
    pub total_runs: usize,
    pub executed_count: usize,
    pub message: String,
}
