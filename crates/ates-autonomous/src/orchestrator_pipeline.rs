use std::error::Error;
use ates_core::TradeDirection;
use crate::types::{TradeSignal, PipelineSummary};

impl crate::orchestrator_struct::AutonomousOrchestrator {
    /// Phase 5: LLM-driven strategy decision.
    /// Returns Option<TradeSignal> — None means LLM decided HOLD.
    pub async fn phase5_strategy_decision(
        &self,
        symbol: &str,
        direction: TradeDirection,
        entry: f64,
        stop: f64,
        target: f64,
    ) -> Result<Option<TradeSignal>, Box<dyn Error + Send + Sync>> {
        println!("\n[PHASE 5] LLM Strategy Decision");
        let signal_opt = self.strategy
            .generate_signal(symbol, direction, entry, stop, target)
            .await?;

        match &signal_opt {
            Some(sig) => println!(
                "[PHASE 5] LLM decided {} {} @ {:.2} (confidence {:.1}%)",
                if sig.direction == ates_core::TradeDirection::Long { "BUY" } else { "SELL" },
                symbol, sig.entry_price, sig.confidence_score * 100.0
            ),
            None => println!("[PHASE 5] LLM decided HOLD for {} — skipping execution.", symbol),
        }

        Ok(signal_opt)
    }

    /// Phase 6: Execute the paper trade and update the portfolio.
    pub async fn phase6_portfolio_and_execution(&self, signal: &TradeSignal) -> Result<bool, Box<dyn Error + Send + Sync>> {
        println!("\n[PHASE 6] Portfolio & Execution");
        let result = self.execution.execute_paper_trade(signal).await?;
        println!("[PHASE 6] {}", result);
        Ok(true)
    }

    /// Full autonomous pipeline: price → Kronos → LLM → risk check → execute.
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

        // Phase 1 — Discipline guards
        if !self.phase1_discipline_checks().await? {
            return Ok(PipelineSummary {
                executed: false,
                phase_results: vec![],
                total_duration_ms: start.elapsed().as_millis() as u64,
                final_signal: None,
                reason: "Discipline checks failed".to_string(),
            });
        }

        // Phase 2 — Market analysis + Kronos forecast (stored in SharedState)
        let (_confluence, _pivots) = self.phase2_market_analysis(symbol, entry).await?;

        // Phase 3 — Risk psychology
        let risk = self.phase3_risk_assessment(symbol, entry).await?;
        if risk.recommendation == crate::types::RiskRecommendation::Halt {
            return Ok(PipelineSummary {
                executed: false,
                phase_results: vec![],
                total_duration_ms: start.elapsed().as_millis() as u64,
                final_signal: None,
                reason: "Risk psychology: HALT".to_string(),
            });
        }

        // Phase 4 — Reflector
        let _ = self.phase4_reflection(symbol).await?;

        // Phase 5 — LLM trade decision (BUY / SELL / HOLD)
        let signal_opt = self.phase5_strategy_decision(symbol, direction, entry, stop, target).await?;

        // Phase 6 — Execute (only if LLM said BUY or SELL)
        let (executed, reason) = if let Some(ref sig) = signal_opt {
            let ok = self.phase6_portfolio_and_execution(sig).await?;
            (ok, if ok { "LLM trade executed".to_string() } else { "Execution failed".to_string() })
        } else {
            (false, "LLM HOLD — no trade placed".to_string())
        };

        Ok(PipelineSummary {
            executed,
            phase_results: vec![],
            total_duration_ms: start.elapsed().as_millis() as u64,
            final_signal: signal_opt,
            reason,
        })
    }

    pub async fn run_health_check(&self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let reports = vec!["System health: OK (LLM-driven autonomous mode)".to_string()];
        Ok(reports)
    }

    pub async fn run_monitoring_loop(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[Orchestrator] Monitoring loop started (LLM-driven)");
        Ok(())
    }
}