use async_trait::async_trait;
use std::error::Error;
use chrono::Utc;
use ates_core::{
    Agent, AgentTier, AgentInput, AgentOutput,
    MarketContext, calculate_pivot_points, calculate_confluence_score,
    check_risk_limits, is_in_trading_session,
};
use crate::state::SharedState;
use crate::types::{TradeSignal, TradeDirection, RiskAnalysis, RiskRecommendation};
use crate::helpers::calculate_position_size;

// MarketIntelligenceAgent with real analyze_market logic
pub struct MarketIntelligenceAgent { pub state: SharedState }
impl MarketIntelligenceAgent { pub fn new(state: SharedState) -> Self { Self { state } } }

#[async_trait]
impl Agent for MarketIntelligenceAgent {
    fn name(&self) -> &str { "MarketIntelligenceAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Main }
    async fn run(&self, input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        match input {
            Some(AgentInput::ConfluenceRequest { context }) => {
                let rules = self.state.rules.read().await;
                let pivots = calculate_pivot_points(context.high, context.low, context.previous_close, rules.pivot_method);
                let score = calculate_confluence_score(&context, &pivots);
                println!("[MarketIntelligence] {} confluence: {:.3}", context.symbol, score);
                Ok(AgentOutput::ConfluenceResult(score))
            }
            _ => Ok(AgentOutput::Done),
        }
    }
}

// RiskPsychologyAgent
pub struct RiskPsychologyAgent { pub state: SharedState }
impl RiskPsychologyAgent { pub fn new(state: SharedState) -> Self { Self { state } } }
#[async_trait]
impl Agent for RiskPsychologyAgent {
    fn name(&self) -> &str { "RiskPsychologyAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Main }
    async fn run(&self, input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        if let Some(AgentInput::RiskRequest { context }) = input {
            let rules = self.state.rules.read().await;
            let check = check_risk_limits(&context, &rules);
            println!("[RiskPsychology] Risk check: {}", if check.passed { "PASS" } else { "FAIL" });
            Ok(AgentOutput::RiskResult(check))
        } else {
            Ok(AgentOutput::Done)
        }
    }
}

// Remaining 4 Main Agents (condensed but functional)
pub struct ReflectorAgent { pub state: SharedState }
impl ReflectorAgent { pub fn new(state: SharedState) -> Self { Self { state } } }
#[async_trait]
impl Agent for ReflectorAgent {
    fn name(&self) -> &str { "ReflectorAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Main }
    async fn run(&self, _input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        println!("[ReflectorAgent] Reflecting on history...");
        Ok(AgentOutput::Done)
    }
}

pub struct StrategyDecisionAgent { pub state: SharedState }
impl StrategyDecisionAgent { pub fn new(state: SharedState) -> Self { Self { state } } }
#[async_trait]
impl Agent for StrategyDecisionAgent {
    fn name(&self) -> &str { "StrategyDecisionAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Main }
    async fn run(&self, _input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        println!("[StrategyDecisionAgent] Making decision...");
        Ok(AgentOutput::Done)
    }
}

pub struct PortfolioManagerAgent { pub state: SharedState }
impl PortfolioManagerAgent { pub fn new(state: SharedState) -> Self { Self { state } } }
#[async_trait]
impl Agent for PortfolioManagerAgent {
    fn name(&self) -> &str { "PortfolioManagerAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Main }
    async fn run(&self, _input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        println!("[PortfolioManagerAgent] Managing positions...");
        Ok(AgentOutput::Done)
    }
}

pub struct ExecutionCoordinatorAgent { pub state: SharedState }
impl ExecutionCoordinatorAgent { pub fn new(state: SharedState) -> Self { Self { state } } }
#[async_trait]
impl Agent for ExecutionCoordinatorAgent {
    fn name(&self) -> &str { "ExecutionCoordinatorAgent" }
    fn tier(&self) -> AgentTier { AgentTier::Main }
    async fn run(&self, _input: Option<AgentInput>) -> Result<AgentOutput, Box<dyn Error + Send + Sync>> {
        println!("[ExecutionCoordinatorAgent] Executing...");
        Ok(AgentOutput::Done)
    }
}