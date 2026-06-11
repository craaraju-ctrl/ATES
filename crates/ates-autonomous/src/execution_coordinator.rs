use async_trait::async_trait;
use std::error::Error;
use chrono::Utc;
use ates_core::{Agent, AgentTier, AgentInput, AgentOutput};
use crate::state::SharedState;
use crate::types::TradeSignal;

pub struct ExecutionCoordinatorAgent {
    pub state: SharedState,
}

impl ExecutionCoordinatorAgent {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }

    async fn execute_paper_trade(&self, signal: &TradeSignal) -> Result<String, Box<dyn Error + Send + Sync>> {
        println!(
            "[ExecutionCoordinator] Executing paper trade: {} {} @ {:.2} | Qty: {:.0}",
            signal.symbol,
            if signal.direction == ates_core::TradeDirection::Long { "BUY" } else { "SELL" },
            signal.entry_price, signal.position_size
        );

        if signal.position_size <= 0.0 {
            return Err("Invalid position size".into());
        }

        println!("[ExecutionCoordinator] Order filled (simulated)");

        let pm = crate::portfolio_manager::PortfolioManagerAgent::new(self.state.clone());
        let _ = pm.add_position(signal).await;

        let exec_log = format!(
            "EXECUTED: {} {} {:.0} @ {:.2} | Stop: {:.2} | Target: {:.2}",
            signal.symbol,
            if signal.direction == ates_core::TradeDirection::Long { "BUY" } else { "SELL" },
            signal.position_size, signal.entry_price, signal.stop_loss, signal.take_profit
        );

        let _ = self.state.memory.store_decision(
            &format!("execution/{}/{}", signal.symbol, Utc::now().timestamp()),
            &exec_log
        );

        Ok(format!("Paper trade executed: {}", exec_log))
    }

    async fn check_and_exit_positions(&self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let portfolio = self.state.portfolio.read().await;
        let mut exits = Vec::new();
        let positions_snapshot = portfolio.open_positions.clone();
        drop(portfolio);

        let pm = crate::portfolio_manager::PortfolioManagerAgent::new(self.state.clone());

        for pos in &positions_snapshot {
            let current_price = pos.current_price;

            let stop_hit = match pos.direction {
                ates_core::TradeDirection::Long => current_price <= pos.stop_loss,
                ates_core::TradeDirection::Short => current_price >= pos.stop_loss,
            };

            let tp_hit = match pos.direction {
                ates_core::TradeDirection::Long => current_price >= pos.take_profit,
                ates_core::TradeDirection::Short => current_price <= pos.take_profit,
            };

            if stop_hit {
                println!("[ExecutionCoordinator] STOP LOSS hit for {} @ {:.2}", pos.symbol, current_price);
                match pm.close_position(&pos.symbol, pos.stop_loss).await {
                    Ok(pnl) => exits.push(format!("{} STOP @ {:.2} P&L: ₹{:.2}", pos.symbol, pos.stop_loss, pnl)),
                    Err(e) => exits.push(format!("{} STOP FAILED: {}", pos.symbol, e)),
                }
            } else if tp_hit {
                println!("[ExecutionCoordinator] TAKE PROFIT hit for {} @ {:.2}", pos.symbol, current_price);
                match pm.close_position(&pos.symbol, pos.take_profit).await {
                    Ok(pnl) => exits.push(format!("{} TP @ {:.2} P&L: ₹{:.2}", pos.symbol, pos.take_profit, pnl)),
                    Err(e) => exits.push(format!("{} TP FAILED: {}", pos.symbol, e)),
                }
            }
        }

        Ok(exits)
    }
}

#[async_trait]
impl Agent for ExecutionCoordinatorAgent {
    fn name(&self) -> &str { "ExecutionCoordinatorAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Main }

    async fn run(&self, input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        match input {
            Some(AgentInput::ConfluenceRequest { context }) => {
                let signal = TradeSignal {
                    symbol: context.symbol.clone(),
                    direction: ates_core::TradeDirection::Long,
                    entry_price: context.current_price,
                    stop_loss: context.current_price * 0.99,
                    take_profit: context.current_price * 1.02,
                    position_size: 10.0,
                    confidence_score: 0.7,
                    confluence_score: 0.7,
                    risk_reward_ratio: 2.0,
                    reasoning: "Auto-generated from execution context".to_string(),
                    timestamp: Utc::now(),
                    session_valid: true,
                    risk_check_passed: true,
                };

                match self.execute_paper_trade(&signal).await {
                    Ok(result) => {
                        println!("[ExecutionCoordinator] {}", result);
                        Ok(AgentOutput::Done)
                    }
                    Err(e) => {
                        println!("[ExecutionCoordinator] Execution failed: {}", e);
                        Ok(AgentOutput::NoOutput)
                    }
                }
            }
            _ => {
                let exits = self.check_and_exit_positions().await?;
                if !exits.is_empty() {
                    for exit in &exits {
                        println!("[ExecutionCoordinator] {}", exit);
                    }
                }
                Ok(AgentOutput::Done)
            }
        }
    }
}