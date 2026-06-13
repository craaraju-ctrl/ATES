// SentimentAnalyzer helper (new skill/tool)
// Analyzes news/sentiment for market intelligence upgrade.
// Research-backed: Integrates with news like in TradingAgents/FINCON for better signals.
// Pure helper methods (no broken Agent impl).

use crate::state::SharedState;

pub struct SentimentAnalyzer {
    pub state: SharedState,
}

impl SentimentAnalyzer {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }

    /// Analyzes sentiment from latest news context.
    pub async fn analyze_sentiment(&self, symbol: &str) -> f64 {
        let news = self.state.latest_news.read().await;
        if let Some(ctx) = news.get(symbol) {
            let summary = &ctx.summary.to_lowercase();
            let pos = ["bull", "up", "gain", "positive", "rise", "buy"].iter().filter(|&w| summary.contains(w)).count() as f64;
            let neg = ["bear", "down", "loss", "negative", "fall", "sell"].iter().filter(|&w| summary.contains(w)).count() as f64;
            let score = ((pos - neg) / (pos + neg + 1.0) + 0.5).clamp(0.0, 1.0);
            return score;
        }
        0.5 // neutral
    }
}
