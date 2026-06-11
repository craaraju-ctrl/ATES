pub mod types;
pub mod state;
pub mod helpers;

pub mod market_intelligence;
pub mod risk_psychology;
pub mod reflector;
pub mod strategy_decision;
pub mod portfolio_manager;
pub mod execution_coordinator;

pub mod risk_calculator;
pub mod pivot_calculator;
pub mod confluence_scorer;

pub mod orchestrator_struct;
pub mod orchestrator_phases;
pub mod orchestrator_pipeline;

pub use orchestrator_struct::AutonomousOrchestrator;

pub async fn main_autonomous() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("ATES Autonomous System Initialized");
    Ok(())
}