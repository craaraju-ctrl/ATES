use std::error::Error;
use ates_core::TradeDirection;
use crate::types::{TradeSignal, PipelineSummary};

impl crate::orchestrator_struct::AutonomousOrchestrator {
    pub async fn phase5_strategy_decision(
        &self,
        symbol: &str,
        direction: TradeDirection,
        entry: f64,
        stop: f64,
        target: f64,
    ) -> Result<Option<TradeSignal>, Box<dyn Error + Send + Sync>> {
        println!("\n[PHASE 5] Strategy Decision");
        let signal = self.strategy.generate_signal(symbol, direction, entry, stop, target).await?;
        Ok(Some(signal))
    }

    pub async fn phase6_portfolio_and_execution(&self, signal: &TradeSignal) -> Result<bool, Box<dyn Error + Send + Sync>> {
        println!("\n[PHASE 6] Portfolio & Execution");
        let result = self.execution.execute_paper_trade(signal).await?;
        println!("[PHASE 6] {}", result);
        Ok(true)
    }

    pub async fn run_full_pipeline(
        &self,
        symbol: &str,
        direction: TradeDirection,
        entry: f64,
        stop: f64,
        target: f64,
    ) -> Result<PipelineSummary, Box<dyn Error + Send + Sync>> {
        let start = std::time::Instant::now();

        println!("\n=== ATES AUTONOMOUS PIPELINE for {} ===", symbol);

        if !self.phase1_discipline_checks().await? {
            return Ok(PipelineSummary { executed: false, phase_results: vec![], total_duration_ms: start.elapsed().as_millis() as u64, final_signal: None, reason: "Discipline failed".to_string() });
        }

        let (_confluence, _pivots) = self.phase2_market_analysis(symbol, entry).await?;
        let risk = self.phase3_risk_assessment(symbol, entry).await?;

        if risk.recommendation == crate::types::RiskRecommendation::Halt {
            return Ok(PipelineSummary { executed: false, phase_results: vec![], total_duration_ms: start.elapsed().as_millis() as u64, final_signal: None, reason: "Risk HALT".to_string() });
        }

        let _ = self.phase4_reflection(symbol).await?;
        let signal_opt = self.phase5_strategy_decision(symbol, direction, entry, stop, target).await?;
        let executed = if let Some(ref sig) = signal_opt {
            self.phase6_portfolio_and_execution(sig).await?
        } else { false };

        Ok(PipelineSummary {
            executed,
            phase_results: vec![],
            total_duration_ms: start.elapsed().as_millis() as u64,
            final_signal: signal_opt,
            reason: if executed { "Executed".to_string() } else { "No signal".to_string() },
        })
    }

    pub async fn run_health_check(&self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let mut reports = Vec::new();
        reports.push("System health: OK (simplified)".to_string());
        Ok(reports)
    }

    pub async fn run_monitoring_loop(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[Orchestrator] Monitoring loop started");
        Ok(())
    }
}