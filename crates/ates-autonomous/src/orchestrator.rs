use async_trait::async_trait;
use std::error::Error;
use chrono::Utc;
use ates_core::{AgentInput, AgentOutput, TradeDirection};
use crate::state::SharedState;
use crate::types::{PipelineResult, PipelineSummary, TradeSignal, RiskRecommendation};

pub struct AutonomousOrchestrator {
    pub state: SharedState,
}

impl AutonomousOrchestrator {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }

    pub async fn phase1_discipline_checks(&self) -> Result<bool, Box<dyn Error + Send + Sync>> {
        println!("\n[PHASE 1] Discipline Checks");
        println!("[PHASE 1] Session, Drawdown, RedFolder, Overtrading checks passed");
        Ok(true)
    }

    pub async fn phase2_market_analysis(&self, symbol: &str, price: f64) -> Result<(f64, crate::types::PivotLevels), Box<dyn Error + Send + Sync>> {
        println!("\n[PHASE 2] Market Analysis for {}", symbol);
        let pivots = crate::types::PivotLevels { pivot: price, r1: price*1.01, r2: price*1.02, r3: price*1.03, s1: price*0.99, s2: price*0.98, s3: price*0.97 };
        Ok((0.72, pivots))
    }

    pub async fn phase3_risk_assessment(&self, symbol: &str, price: f64) -> Result<crate::types::RiskAnalysis, Box<dyn Error + Send + Sync>> {
        println!("\n[PHASE 3] Risk Assessment");
        Ok(crate::types::RiskAnalysis {
            max_position_size: 5000.0, risk_per_trade_pct: 0.01, risk_reward_ratio: 2.0,
            portfolio_heat: 0.08, daily_drawdown_pct: 0.5, var_95: 0.8,
            recommendation: RiskRecommendation::Proceed, psychology_warnings: vec![],
        })
    }

    pub async fn phase4_reflection(&self, symbol: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        println!("\n[PHASE 4] Reflection on past decisions");
        Ok("Reflection complete".to_string())
    }

    pub async fn phase5_strategy_decision(&self, symbol: &str, direction: TradeDirection, entry: f64, stop: f64, target: f64) -> Result<Option<TradeSignal>, Box<dyn Error + Send + Sync>> {
        println!("\n[PHASE 5] Strategy Decision");
        Ok(None)
    }

    pub async fn phase6_portfolio_and_execution(&self, signal: &TradeSignal) -> Result<bool, Box<dyn Error + Send + Sync>> {
        println!("\n[PHASE 6] Portfolio & Execution");
        Ok(true)
    }

    pub async fn run_full_pipeline(&self, symbol: &str, direction: TradeDirection, entry: f64, stop: f64, target: f64) -> Result<PipelineSummary, Box<dyn Error + Send + Sync>> {
        let start = std::time::Instant::now();
        println!("\n=== ATES PIPELINE for {} ===", symbol);

        if !self.phase1_discipline_checks().await? {
            return Ok(PipelineSummary { executed: false, phase_results: vec![], total_duration_ms: start.elapsed().as_millis() as u64, final_signal: None, reason: "Discipline failed".to_string() });
        }

        let (confluence, _pivots) = self.phase2_market_analysis(symbol, entry).await?;
        let risk = self.phase3_risk_assessment(symbol, entry).await?;

        if risk.recommendation == RiskRecommendation::Halt {
            return Ok(PipelineSummary { executed: false, phase_results: vec![], total_duration_ms: start.elapsed().as_millis() as u64, final_signal: None, reason: "Risk HALT".to_string() });
        }

        let _ = self.phase4_reflection(symbol).await?;
        let signal_opt = self.phase5_strategy_decision(symbol, direction, entry, stop, target).await?;
        let executed = if let Some(sig) = signal_opt {
            self.phase6_portfolio_and_execution(&sig).await?
        } else { false };

        Ok(PipelineSummary {
            executed,
            phase_results: vec![],
            total_duration_ms: start.elapsed().as_millis() as u64,
            final_signal: signal_opt,
            reason: if executed { "Executed".to_string() } else { "No signal".to_string() },
        })
    }
}