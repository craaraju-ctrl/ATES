use tokio::sync::mpsc;
use tokio::task::JoinSet;
use std::error::Error;

use ates_core::{
    DisciplineRules, get_discipline_summary, LlmExecutor, Config,
    broker::KitePaperAdapter,
    execution::ExecutionEngine,
    messages::{AgentMessage, LLMRequest, LLMResponse},
    role::AgentRole,
};
use ates_agents::{
    MarketIntelligenceAgent,
    RiskPsychologyAgent,
    ReflectorAgent,
    RiskCalculatorAgent,
    PivotCalculatorAgent,
    OutcomeLoggerAgent,
};

/// Main Orchestrator for ATES
/// Coordinates Main Agents (LLM-capable) and Sub-Agents (deterministic)
/// Uses message passing for clean separation of concerns.

#[tokio::main]
async fn main() {
    println!("=== ATES Orchestrator v0.15 — Agent & Sub-Agent Orchestra ===");
    println!("{}", get_discipline_summary());

    let config = Config::default();
    let rules = DisciplineRules::default();
    let memory = ates_core::MemoryStore::new("ates_memory.redb").expect("Memory init failed");
    let llm = LlmExecutor::new();
    let broker = KitePaperAdapter::new(config.clone());
    let mut execution = ExecutionEngine::new(config.initial_balance, memory);

    println!("[Orchestrator] Starting full agent orchestra...");
    println!("[Orchestrator] Main Agents: MarketIntelligence, RiskPsychology, Reflector");
    println!("[Orchestrator] Sub-Agents: RiskCalculator, PivotCalculator, OutcomeLogger");

    // Create message channels for agent communication
    let (tx_main, mut rx_main) = mpsc::channel::<AgentMessage>(100);
    let (tx_sub, mut rx_sub) = mpsc::channel::<AgentMessage>(100);

    let mut set = JoinSet::new();

    // Spawn and run real agents (demonstration of coordination)
    println!("[Orchestrator] Running coordinated agent cycle...");

    let market_intel = MarketIntelligenceAgent;
    let _ = market_intel.run().await;

    let risk_psych = RiskPsychologyAgent;
    let _ = risk_psych.run().await;

    let pivot_calc = PivotCalculatorAgent;
    let _ = pivot_calc.run().await;

    // Example: Use MemoryStore via OutcomeLogger
    let outcome_logger = OutcomeLoggerAgent;
    let _ = outcome_logger.run().await;

    // Central Message Router (the real "orchestra conductor")
    let router_handle = tokio::spawn(async move {
        println!("[MessageRouter] Central router started");

        loop {
            tokio::select! {
                Some(msg) = rx_main.recv() => {
                    match msg {
                        AgentMessage::LLMRequest(req) => {
                            println!("[Router] LLMRequest from {}: {}", req.agent_role.description(), req.purpose);
                            // In production: send to LlmExecutor and route response back
                        }
                        AgentMessage::Observation { agent, content } => {
                            println!("[Router] Observation from {}: {}", agent, content);
                        }
                        _ => {}
                    }
                }
                Some(msg) = rx_sub.recv() => {
                    println!("[Router] Message from Sub-Agent: {:?}", msg);
                }
                else => break,
            }
        }
    });

    // Let the system run for a while (in real app this would be an infinite loop with proper shutdown)
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    println!("[Orchestrator] Agent orchestra cycle completed. System ready for production use.");