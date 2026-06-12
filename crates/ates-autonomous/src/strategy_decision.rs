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

    /// Generate a trade signal for a symbol by:
    /// 1. Reading the Kronos forecast stored by Phase 2 (MarketIntelligenceAgent)
    /// 2. Asking the Ollama LLM to decide BUY / SELL / HOLD with entry, SL, TP
    /// 3. Validating the decision against discipline rules
    /// 4. Returning the populated TradeSignal (or None if HOLD)
    pub async fn generate_signal(
        &self,
        symbol: &str,
        _direction: ates_core::TradeDirection,   // initial direction hint from price momentum
        entry: f64,
        _stop: f64,
        _target: f64,
    ) -> Result<Option<TradeSignal>, Box<dyn Error + Send + Sync>> {
        let rules     = self.state.rules.read().await;
        let portfolio = self.state.portfolio.read().await;

        let context = MarketContext {
            symbol: symbol.to_string(),
            current_price: entry,
            high:  entry * 1.01,
            low:   entry * 0.99,
            previous_close: entry * 0.998,
            timestamp: Utc::now(),
            daily_pnl: portfolio.daily_pnl,
            consecutive_losses: portfolio.consecutive_losses,
            is_red_folder_day: false,
            trend_direction: None,
        };

        let pivots     = calculate_pivot_points(context.high, context.low, context.previous_close, rules.pivot_method);
        let confluence = calculate_confluence_score(&context, &pivots);
        let session    = get_indian_session_info(Utc::now());

        // Pull Kronos forecast from SharedState
        let forecast_summary = {
            let last = self.state.last_forecast.read().await;
            match last.as_ref() {
                Some(v) => v["summary"].as_str().unwrap_or("No forecast summary").to_string(),
                None    => "Kronos unavailable".to_string(),
            }
        };

        // Determine trend label for the prompt
        let trend_label = {
            let regime = self.state.market_regime.read().await;
            match *regime {
                Some(crate::types::MarketRegime::TrendingBull) => "Bullish",
                Some(crate::types::MarketRegime::TrendingBear) => "Bearish",
                Some(crate::types::MarketRegime::Ranging)      => "Ranging",
                _                                               => "Neutral",
            }
        };

        let portfolio_heat: f64 = {
            let total_risk: f64 = portfolio.open_positions.iter().map(|p| p.risk_amount).sum();
            if portfolio.total_equity > 0.0 { total_risk / portfolio.total_equity } else { 0.0 }
        };
        let consecutive_losses = portfolio.consecutive_losses;

        // Release lock before async LLM call
        drop(portfolio);
        drop(rules);

        // ── Ask the LLM for a trade decision ──────────────────────────────────
        let decision = self.state.llm.ask_for_trade_decision(
            symbol, entry, confluence, trend_label,
            pivots.pivot, pivots.r1, pivots.s1,
            &forecast_summary,
            portfolio_heat,
            session.market_open,
            consecutive_losses,
        ).await;

        // Store LLM reasoning for debugging / UI
        {
            let mut reason_store = self.state.last_llm_reason.write().await;
            *reason_store = format!("[{}] {}: {}", symbol, decision.action, decision.reason);
        }

        // ── Handle HOLD ────────────────────────────────────────────────────────
        if decision.action == "HOLD" {
            println!("[StrategyDecision] 🤚 LLM HOLD for {} — {}", symbol, decision.reason);
            return Ok(None);
        }

        // ── Map LLM action to TradeDirection ───────────────────────────────────
        let trade_direction = if decision.action == "BUY" {
            ates_core::TradeDirection::Long
        } else {
            ates_core::TradeDirection::Short
        };

        let entry_price = decision.entry;
        let stop_loss   = decision.sl;
        let take_profit = decision.tp;

        // ── Re-validate against discipline rules ───────────────────────────────
        let rules2   = self.state.rules.read().await;
        let portfolio2 = self.state.portfolio.read().await;
        let discipline = validate_trade_setup(&context, &rules2);

        let adjusted_risk = if portfolio2.consecutive_losses >= 2 {
            rules2.max_risk_per_trade * 0.5
        } else {
            rules2.max_risk_per_trade
        };

        let position_size = calculate_position_size(
            portfolio2.total_equity,
            adjusted_risk,
            entry_price,
            stop_loss,
        );

        let risk_reward = calculate_risk_reward(entry_price, stop_loss, take_profit, trade_direction);

        let session_info = get_indian_strategy_session_info(Utc::now());

        let confidence = if discipline.passed {
            let base = confluence;
            let rr_bonus = (risk_reward / 3.0).min(0.2);
            (base + rr_bonus).min(1.0)
        } else {
            confluence * 0.5   // lower confidence when discipline fails but LLM still decided to trade
        };

        let signal = TradeSignal {
            symbol: symbol.to_string(),
            direction: trade_direction,
            entry_price,
            stop_loss,
            take_profit,
            position_size,
            confidence_score: confidence,
            confluence_score: confluence,
            risk_reward_ratio: risk_reward,
            reasoning: format!(
                "LLM[{}]: {} | Confluence: {:.2} | R:R {:.1}:1 | Discipline: {} | Session: {}",
                decision.action,
                decision.reason,
                confluence,
                risk_reward,
                if discipline.passed { "PASS" } else { "FAIL" },
                if session_info { "OPEN" } else { "CLOSED" },
            ),
            timestamp: Utc::now(),
            session_valid: session_info,
            risk_check_passed: discipline.passed,
        };

        println!(
            "[StrategyDecision] 🤖 LLM {} {} @ {:.2} | Confidence: {:.1}% | {}",
            decision.action, symbol, entry_price,
            confidence * 100.0, decision.reason
        );

        // Store signal in history
        {
            let mut signals = self.state.last_signals.write().await;
            signals.push(signal.clone());
            if signals.len() > 100 { signals.remove(0); }
        }

        // Store in memory
        let _ = self.state.memory.store_decision(
            &format!("signal/{}/{}", symbol, Utc::now().timestamp()),
            &signal.reasoning,
        );

        Ok(Some(signal))
    }
}

/// Returns whether the market session is valid for the given time.
fn get_indian_strategy_session_info(now: chrono::DateTime<Utc>) -> bool {
    use chrono::Timelike;
    let ist = now + chrono::Duration::hours(5) + chrono::Duration::minutes(30);
    let hour = ist.hour();
    let min  = ist.minute();
    let time_mins = hour * 60 + min;
    // NSE: 9:15 - 15:30 IST
    time_mins >= 9 * 60 + 15 && time_mins <= 15 * 60 + 30
}

#[async_trait]
impl Agent for StrategyDecisionAgent {
    fn name(&self) -> &str { "StrategyDecisionAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Main }

    async fn run(&self, _input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        println!("[StrategyDecisionAgent] LLM-driven signal generation — call generate_signal() directly.");
        Ok(AgentOutput::Done)
    }
}