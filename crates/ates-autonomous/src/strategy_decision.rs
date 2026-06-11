use async_trait::async_trait;
use std::error::Error;
use chrono::Utc;
use ates_core::{
    Agent, AgentTier, AgentInput, AgentOutput,
    MarketContext, calculate_pivot_points, calculate_confluence_score,
    validate_trade_setup,
};
use crate::state::SharedState;
use crate::types::TradeSignal;
use crate::helpers::{calculate_position_size, calculate_risk_reward, get_indian_session_info};

pub struct StrategyDecisionAgent {
    pub state: SharedState,
}

impl StrategyDecisionAgent {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }

    pub async fn generate_signal(
        &self,
        symbol: &str,
        direction: ates_core::TradeDirection,
        entry: f64,
        stop: f64,
        target: f64,
    ) -> Result<TradeSignal, Box<dyn Error + Send + Sync>> {
        let rules = self.state.rules.read().await;
        let portfolio = self.state.portfolio.read().await;

        let context = MarketContext {
            symbol: symbol.to_string(),
            current_price: entry,
            high: entry * 1.01,
            low: entry * 0.99,
            previous_close: entry * 0.998,
            timestamp: Utc::now(),
            daily_pnl: portfolio.daily_pnl,
            consecutive_losses: portfolio.consecutive_losses,
            is_red_folder_day: false,
            trend_direction: None,
        };

        let discipline = validate_trade_setup(&context, &rules);
        let pivots = calculate_pivot_points(context.high, context.low, context.previous_close, rules.pivot_method);
        let confluence = calculate_confluence_score(&context, &pivots);

        let adjusted_risk = if portfolio.consecutive_losses >= 2 {
            rules.max_risk_per_trade * 0.5
        } else {
            rules.max_risk_per_trade
        };

        let position_size = calculate_position_size(
            portfolio.total_equity,
            adjusted_risk,
            entry,
            stop,
        );

        let risk_reward = calculate_risk_reward(entry, stop, target, direction);
        let session_info = get_indian_session_info(Utc::now());

        let confidence = if discipline.passed {
            let base = confluence;
            let rr_bonus = (risk_reward / 3.0).min(0.2);
            (base + rr_bonus).min(1.0)
        } else {
            0.0
        };

        let signal = TradeSignal {
            symbol: symbol.to_string(),
            direction,
            entry_price: entry,
            stop_loss: stop,
            take_profit: target,
            position_size,
            confidence_score: confidence,
            confluence_score: confluence,
            risk_reward_ratio: risk_reward,
            reasoning: format!(
                "Confluence: {:.2} | R:R {:.1}:1 | Discipline: {} | Session: {}",
                confluence, risk_reward,
                if discipline.passed { "PASS" } else { "FAIL" },
                if session_info.market_open { "OPEN" } else { "CLOSED" }
            ),
            timestamp: Utc::now(),
            session_valid: session_info.market_open,
            risk_check_passed: discipline.passed,
        };

        println!(
            "[StrategyDecision] {} {} @ {:.2} | Confidence: {:.1}%",
            symbol,
            if direction == ates_core::TradeDirection::Long { "LONG" } else { "SHORT" },
            entry, confidence * 100.0
        );

        {
            let mut signals = self.state.last_signals.write().await;
            signals.push(signal.clone());
            if signals.len() > 100 { signals.remove(0); }
        }

        Ok(signal)
    }
}

#[async_trait]
impl Agent for StrategyDecisionAgent {
    fn name(&self) -> &str { "StrategyDecisionAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Main }

    async fn run(&self, _input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        println!("[StrategyDecisionAgent] Generating trade signal...");
        Ok(AgentOutput::Done)
    }
}