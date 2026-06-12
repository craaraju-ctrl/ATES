use async_trait::async_trait;
use std::error::Error;
use chrono::Utc;
use ates_core::{
    Agent, AgentTier, AgentInput, AgentOutput,
    MarketContext, calculate_pivot_points, calculate_confluence_score,
    is_in_trading_session,
    KronosClient, KronosForecastRequest, OhlcvBar,
    TrendDirection,
};
use crate::state::SharedState;

pub struct MarketIntelligenceAgent {
    pub state: SharedState,
}

impl MarketIntelligenceAgent {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }

    /// Run Kronos forecast + pivot/confluence analysis for a symbol.
    /// Stores the full Kronos forecast JSON in SharedState.last_forecast for Phase 5 (LLM).
    pub async fn analyze_market(&self, symbol: &str, price: f64) -> Result<(f64, ates_core::PivotLevels), Box<dyn Error + Send + Sync>> {
        let rules = self.state.rules.read().await;

        let high = price * 1.015;
        let low  = price * 0.985;
        let prev_close = price * 0.998;

        // --- Kronos Forecast Service ---
        let kronos_client = KronosClient::new(self.state.config.kronos_service_url.clone());

        let sample_ohlcv = vec![
            OhlcvBar {
                timestamp: Utc::now().to_rfc3339(),
                open:  prev_close,
                high,
                low,
                close: price,
                volume: 100_000.0,
            }
        ];

        let forecast_req = KronosForecastRequest {
            symbol: symbol.to_string(),
            ohlcv:  sample_ohlcv,
            pred_len:     5,
            temperature:  0.8,
            top_p:        0.9,
            sample_count: 1,
        };

        let mut trend_direction = None;
        let mut forecast_summary = String::from("No forecast available");

        match kronos_client.forecast(forecast_req).await {
            Ok(resp) => {
                // Summarise the 5-bar forecast for the LLM prompt
                let closes: Vec<f64> = resp.forecasts.iter()
                    .filter_map(|f| f.get("close").and_then(|c| c.as_f64()))
                    .collect();

                if let Some(&pred_close) = closes.last() {
                    println!(
                        "[MarketIntelligence] Kronos predicted close for {}: {:.2} (current: {:.2})",
                        symbol, pred_close, price
                    );

                    trend_direction = Some(if pred_close > price {
                        TrendDirection::Bullish
                    } else if pred_close < price {
                        TrendDirection::Bearish
                    } else {
                        TrendDirection::Neutral
                    });

                    let pct_change = (pred_close - price) / price * 100.0;
                    forecast_summary = format!(
                        "Predicts {:.2} in 5 bars ({:+.2}%). Closes: {}",
                        pred_close,
                        pct_change,
                        closes.iter().map(|c| format!("{:.2}", c)).collect::<Vec<_>>().join(", ")
                    );
                }

                // Store full forecast JSON for Phase 5 (StrategyDecisionAgent)
                {
                    let mut last = self.state.last_forecast.write().await;
                    *last = Some(serde_json::json!({
                        "symbol": symbol,
                        "forecasts": resp.forecasts,
                        "summary": forecast_summary.clone(),
                    }));
                }
            }
            Err(e) => {
                println!("[MarketIntelligence] Kronos call failed: {}. Defaulting to Neutral.", e);
                let mut last = self.state.last_forecast.write().await;
                *last = None;
            }
        }

        let context = MarketContext {
            symbol: symbol.to_string(),
            current_price: price,
            high,
            low,
            previous_close: prev_close,
            timestamp: Utc::now(),
            daily_pnl: 0.0,
            consecutive_losses: 0,
            is_red_folder_day: false,
            trend_direction,
        };

        let pivots    = calculate_pivot_points(high, low, prev_close, rules.pivot_method);
        let confluence = calculate_confluence_score(&context, &pivots);
        let session_valid = is_in_trading_session(Utc::now(), &rules);

        println!(
            "[MarketIntelligence] {} @ {:.2} | Pivot: {:.2} | R1: {:.2} | S1: {:.2} | Confluence: {:.2}% | Session: {} | {}",
            symbol, price, pivots.pivot, pivots.r1, pivots.s1,
            confluence * 100.0,
            if session_valid { "VALID" } else { "INVALID" },
            forecast_summary
        );

        let _ = self.state.memory.store_decision(
            &format!("market/{}/{}", symbol, Utc::now().timestamp()),
            &format!("price={:.2},confluence={:.3},trend={:?}", price, confluence, trend_direction),
        );

        {
            let mut regime = self.state.market_regime.write().await;
            *regime = Some(if confluence > 0.7 {
                crate::types::MarketRegime::TrendingBull
            } else if confluence < 0.4 {
                crate::types::MarketRegime::TrendingBear
            } else {
                crate::types::MarketRegime::Ranging
            });
        }

        Ok((confluence, pivots))
    }
}

#[async_trait]
impl Agent for MarketIntelligenceAgent {
    fn name(&self) -> &str { "MarketIntelligenceAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Main }

    async fn run(&self, input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        match input {
            Some(AgentInput::ConfluenceRequest { context }) => {
                let (confluence, _) = self.analyze_market(&context.symbol, context.current_price).await?;
                Ok(AgentOutput::ConfluenceResult(confluence))
            }
            Some(AgentInput::PivotRequest { high, low, close }) => {
                let rules = self.state.rules.read().await;
                let pivots = calculate_pivot_points(high, low, close, rules.pivot_method);
                println!("[MarketIntelligence] Pivot levels calculated on request");
                Ok(AgentOutput::PivotResult(pivots))
            }
            _ => {
                let (confluence, _) = self.analyze_market("NIFTY", 24500.0).await?;
                Ok(AgentOutput::ConfluenceResult(confluence))
            }
        }
    }
}