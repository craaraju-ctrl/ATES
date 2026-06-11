use async_trait::async_trait;
use std::error::Error;

use ates_core::{Agent, AgentTier};

pub struct ConfluenceScorerAgent;

#[async_trait]
impl Agent for ConfluenceScorerAgent {
    fn name(&self) -> &str {
        "ConfluenceScorerAgent"
    }

    fn tier(&self) -> AgentTier {
        AgentTier::Sub
    }

    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[{}] Calculating confluence score using pivots, trend, and volume...", self.name());

        // Example context + pivots (real version would receive these via messages)
        let context = ates_core::MarketContext {
            symbol: "NIFTY".to_string(),
            current_price: 24380.0,
            high: 24500.0,
            low: 24200.0,
            previous_close: 24350.0,
            timestamp: chrono::Utc::now(),
            daily_pnl: 1200.0,
            consecutive_losses: 1,
            is_red_folder_day: false,
            trend_direction: Some(ates_core::TrendDirection::Bullish),
        };

        let pivots = ates_core::calculate_pivot_points(24500.0, 24200.0, 24350.0, ates_core::PivotMethod::Classic);
        let score = ates_core::calculate_confluence_score(&context, &pivots);

        println!("   Confluence Score: {:.2}", score);

        Ok(())
    }
}